use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
};

use crate::{
    approximate_element_count, approximate_false_positive_probability, bitset::Bitset,
    optimal_bit_count, optimal_number_of_hashers, BloomFilter, BloomFilterData,
};

pub type KMBloomFilter<H1, H2> = SeededKMBloomFilter<H1, H2, (), ()>;

/// In order to create hash functions with seeds, this trait needs to be implemented for the given hash function.
/// *H* is the hasher to be created. *S* is the type of the seed needed to create *H*.
///
/// # Examples
/// ```
/// use bloom_filter_simple::HasherBuilder;
/// use ahash::AHasher;
///
/// struct AHasherBuilder;
///
/// impl HasherBuilder<AHasher, (u128, u128)> for AHasherBuilder {
///     fn new_with_seed(seed: (u128, u128)) -> AHasher {
///         AHasher::new_with_keys(seed.0, seed.1)
///     }
/// }
/// ```
pub trait HasherBuilder<H, S>
where
    H: Hasher + Clone,
    S: Clone + PartialEq,
{
    fn new_with_seed(seed: S) -> H;
}

/// Bloom filter implementation using the improvements described by Kirsch and Mitzenmacher:
///
/// > Kirsch A., Mitzenmacher M. (2006) Less Hashing, Same Performance: Building a Better Bloom Filter.
/// In: Azar Y., Erlebach T. (eds) Algorithms – ESA 2006. ESA 2006. Lecture Notes in Computer Science, vol 4168.
/// Springer, Berlin, Heidelberg. https://doi.org/10.1007/11841036_42
///
/// # Examples
/// ```
/// use bloom_filter_simple::{BloomFilter,KMBloomFilter};
/// use ahash::AHasher;
/// use std::collections::hash_map::DefaultHasher;
///
/// fn main() {
///     // We plan on storing at most 10,000 elements
///     let desired_capacity = 10_000;
///     // We want to assure that the chance of a false positive is less than 0.0001.
///     let desired_fp_probability = 0.0001;
///
///     // We initialize a new KMBloomFilter by specifying the desired Hashers as type parameters.
///     let mut filter: KMBloomFilter<AHasher, DefaultHasher> = KMBloomFilter::new(desired_capacity, desired_fp_probability);
///
///     // You can insert any type implementing the Hash trait. The bloom filter does not store the
///     // inserted elements but only their hashes. Hence, there is no transfer of ownership required.
///     filter.insert(&5i32);
///     filter.insert(&"Some text");
///     filter.insert(&10_000usize);
///
///     // You can check whether a value has been inserted into the filter before.
///     assert_eq!(false, filter.contains(&3));
///     assert_eq!(true, filter.contains(&5));
///     assert_eq!(true, filter.contains(&"Some text"));
/// }
/// ```
#[derive(Clone)]
pub struct SeededKMBloomFilter<H1, H2, S1, S2>
where
    H1: Hasher + Clone,
    H2: Hasher + Clone,
    S1: Clone + PartialEq,
    S2: Clone + PartialEq,
{
    number_of_hashers: usize,
    bitset: Bitset,
    bits_per_hasher: usize,
    hasher1: H1,
    hasher2: H2,
    seed1: Option<S1>,
    seed2: Option<S2>,
}

impl<H1, H2, S1, S2> SeededKMBloomFilter<H1, H2, S1, S2>
where
    H1: Hasher + Clone,
    H2: Hasher + Clone,
    S1: Clone + PartialEq,
    S2: Clone + PartialEq,
{
    /// Initialize a new instance of SeededKMBloomFilter that guarantees that the false positive rate
    /// is less than *desired_false_positive_probability* for up to *desired_capacity*
    /// elements. The given seeds *seed1* and *seed2* are used to initialize the two used hash functions (*H1*, *H2*).
    ///
    /// SeededKMBloomFilter uses two seeded hash functions *H1* and *H2* with seed types *S1* and *S2* to simulate an arbitrary number of hash
    /// functions. *H1*, *H2*, *S1* and *S2* are specified as type parameters (see examples): SeededKMBloomFilter<H1, H2, S1, S2>.
    ///
    /// ***You have to either use two different hash functions for *H1* and *H2* or at least different seeds!***
    ///
    /// # Panics
    ///
    /// Panics if desired_capacity == 0
    ///
    /// # Examples
    /// ```
    /// use bloom_filter_simple::{BloomFilter, SeededKMBloomFilter, HasherBuilder};
    /// use ahash::AHasher;
    ///
    /// struct AHasherBuilder;
    ///
    /// impl HasherBuilder<AHasher, (u128, u128)> for AHasherBuilder {
    /// fn new_with_seed(seed: (u128, u128)) -> AHasher {
    ///         AHasher::new_with_keys(seed.0, seed.1)
    ///     }
    /// }
    ///
    /// fn main() {
    ///     // We plan on storing at most 10,000 elements
    ///     let desired_capacity = 10_000;
    ///     // We want to assure that the chance of a false positive is less than 0.0001.
    ///     let desired_fp_probability = 0.0001;
    ///
    ///     // We initialize a new SeededKMBloomFilter by specifying the desired Hashers as type parameters
    ///     let mut filter = SeededKMBloomFilter::new_with_seeds::<AHasherBuilder, AHasherBuilder>(desired_capacity, desired_fp_probability, (1,1), (2,2));
    /// }
    /// ```
    pub fn new_with_seeds<B1, B2>(
        desired_capacity: usize,
        desired_false_positive_probability: f64,
        seed1: S1,
        seed2: S2,
    ) -> Self
    where
        B1: HasherBuilder<H1, S1>,
        B2: HasherBuilder<H2, S2>,
    {
        if desired_capacity == 0 {
            panic!("an empty bloom filter is not defined");
        }
        let bit_count = optimal_bit_count(desired_capacity, desired_false_positive_probability);
        let number_of_hashers = optimal_number_of_hashers(desired_capacity, bit_count);
        let bits_per_hasher = (bit_count as f64 / number_of_hashers as f64).ceil() as usize;
        Self {
            bitset: Bitset::new(bits_per_hasher * number_of_hashers),
            number_of_hashers,
            bits_per_hasher,
            hasher1: B1::new_with_seed(seed1.clone()),
            hasher2: B2::new_with_seed(seed2.clone()),
            seed1: Some(seed1),
            seed2: Some(seed2),
        }
    }
    /// Approximate number of elements stored.
    /// Approximation technique taken from Wikipedia:
    /// > Wikipedia, ["Bloom filter"](https://en.wikipedia.org/wiki/Bloom_filter#Approximating_the_number_of_items_in_a_Bloom_filter) [Accessed: 02.12.2020]
    pub fn approximate_element_count(&self) -> f64 {
        approximate_element_count(
            self.number_of_hashers,
            self.bits_per_hasher,
            self.bitset.count_ones(),
        )
    }

    /// Return the current approximate false positive probability which depends on the current
    /// number of elements in the filter.
    ///
    /// The probability is given as a value in the interval [0,1]
    /// Approximation technique taken from Sagi Kedmi:
    /// > S. Kedmi, ["Bloom Filters for the Perplexed"](https://sagi.io/bloom-filters-for-the-perplexed/), July 2017 [Accessed: 02.12.2020]
    pub fn approximate_current_false_positive_probability(&self) -> f64 {
        approximate_false_positive_probability(
            self.number_of_hashers,
            self.bits_per_hasher,
            self.approximate_element_count(),
        )
    }

    /// Checks whether two bloom filters were created with the same desired capacity and desired false
    /// positive probability.
    pub fn eq_configuration(&self, other: &Self) -> bool {
        self.number_of_hashers == other.number_of_hashers
            && self.bits_per_hasher == other.bits_per_hasher
            && self.seed1 == other.seed1
            && self.seed2 == other.seed2
    }

    fn index(i: usize, bits_per_hash: usize, hash_a: u64, hash_b: u64) -> usize {
        i * bits_per_hash
            + hash_a.wrapping_add((i as u64).wrapping_mul(hash_b)) as usize % bits_per_hash
    }

    /// Creates a intersection of this bloom filter and 'other', which means 'contains' of the resulting
    /// bloom filter will always return true for elements inserted both in this bloom filter and in
    /// 'other' before creation.
    /// The false positive probability of the resulting bloom filter is at most the false positive
    /// probability of 'other' or 'self'.
    /// The false positive probability of the resulting bloom filter may be bigger than the false
    /// positive probability of a new empty bloom filter with the intersecting elements inserted.
    /// The functions 'approximate_current_false_positive_probability' and 'approximate_element_count'
    /// called on the resulting bloom filter may return too big approximations.
    ///
    /// # Panics
    ///
    /// Panics if the desired capacity or desired false positive probability of 'self' and 'other'
    /// differ.
    ///
    /// # Examples
    ///
    /// Intersection of two bloom filters with the same configuration.
    /// ```
    /// use bloom_filter_simple::{BloomFilter,KMBloomFilter};
    /// use ahash::AHasher;
    /// use std::collections::hash_map::DefaultHasher;
    ///
    /// fn main() {
    ///     // The configuration of both bloom filters has to be the same
    ///     let desired_capacity = 10_000;
    ///     let desired_fp_probability = 0.0001;
    ///
    ///     // We initialize two new SeededKMBloomFilter
    ///     let mut filter_one: KMBloomFilter<AHasher, DefaultHasher> = KMBloomFilter::new(
    ///         desired_capacity,
    ///         desired_fp_probability
    ///     );
    ///
    ///     let mut filter_two: KMBloomFilter<AHasher, DefaultHasher> = KMBloomFilter::new(
    ///         desired_capacity,
    ///         desired_fp_probability
    ///     );
    ///
    ///     // Insert elements into the first filter
    ///     filter_one.insert(&0);
    ///     filter_one.insert(&1);
    ///
    ///     // Insert elements into the second filter
    ///     filter_two.insert(&1);
    ///     filter_two.insert(&2);
    ///     
    ///     // Now we retrieve the intersection of both filters
    ///     let filter_intersection = filter_one.intersect(&filter_two);
    ///
    ///     // The intersection will return true for a 'contains' check for the elements inserted
    ///     // previously into both constituent filters.
    ///     assert_eq!(false, filter_intersection.contains(&0));
    ///     assert_eq!(true, filter_intersection.contains(&1));
    ///     assert_eq!(false, filter_intersection.contains(&2));
    /// }
    /// ```
    pub fn intersect(&self, other: &Self) -> Self {
        if !self.eq_configuration(other) {
            panic!("unable to intersect k-m bloom filters with different configurations");
        }
        Self {
            number_of_hashers: self.number_of_hashers,
            bitset: self.bitset.intersect(&other.bitset),
            bits_per_hasher: self.bits_per_hasher,
            hasher1: self.hasher1.clone(),
            hasher2: self.hasher2.clone(),
            seed1: self.seed1.clone(),
            seed2: self.seed2.clone(),
        }
    }

    fn generate_hashes<T>(&self, data: &T) -> (u64, u64)
    where
        T: Hash,
    {
        let mut hasher = self.hasher1.clone();
        data.hash(&mut hasher);
        let hash_a = hasher.finish();

        let mut hasher = self.hasher2.clone();
        data.hash(&mut hasher);
        let hash_b = hasher.finish();

        (hash_a, hash_b)
    }
}

impl<H1, H2, S1, S2> SeededKMBloomFilter<H1, H2, S1, S2>
where
    H1: Hasher + Default + Clone,
    H2: Hasher + Default + Clone,
    S1: Clone + PartialEq,
    S2: Clone + PartialEq,
{
    /// Initialize a new instance of SeededKMBloomFilter that guarantees that the false positive rate
    /// is less than *desired_false_positive_probability* for up to *desired_capacity*
    /// elements.
    ///
    /// SeededKMBloomFilter uses two hash functions *H1* and *H2* to simulate an arbitrary number of hash
    /// functions. *H1* and *H2* are specified as type parameters (see examples): SeededKMBloomFilter<H1, H2, S1, S2>.
    /// In this case *S1* and *S2* are not used, since we use default values for types *H1* and *H2*.
    ///
    /// ***You have to use two different hash functions for *H1* and *H2*!***
    ///
    /// # Panics
    ///
    /// Panics if desired_capacity == 0
    ///
    /// # Examples
    /// ```
    /// use bloom_filter_simple::{BloomFilter,KMBloomFilter};
    /// use ahash::AHasher;
    /// use std::collections::hash_map::DefaultHasher;
    ///
    /// fn main() {
    ///     // We plan on storing at most 10,000 elements
    ///     let desired_capacity = 10_000;
    ///     // We want to assure that the chance of a false positive is less than 0.0001.
    ///     let desired_fp_probability = 0.0001;
    ///
    ///     // We initialize a new SeededKMBloomFilter by specifying the desired Hashers as type parameters
    ///     let mut filter: KMBloomFilter<AHasher, DefaultHasher> =
    ///         KMBloomFilter::new(desired_capacity, desired_fp_probability);
    /// }
    /// ```
    pub fn new(desired_capacity: usize, desired_false_positive_probability: f64) -> Self {
        if desired_capacity == 0 {
            panic!("an empty bloom filter is not defined");
        }
        let bit_count = optimal_bit_count(desired_capacity, desired_false_positive_probability);
        let number_of_hashers = optimal_number_of_hashers(desired_capacity, bit_count);
        let bits_per_hasher = (bit_count as f64 / number_of_hashers as f64).ceil() as usize;
        Self {
            bitset: Bitset::new(bits_per_hasher * number_of_hashers),
            number_of_hashers,
            bits_per_hasher,
            hasher1: H1::default(),
            hasher2: H2::default(),
            seed1: None,
            seed2: None,
        }
    }
}

impl<H1, H2, S1, S2> Debug for SeededKMBloomFilter<H1, H2, S1, S2>
where
    H1: Hasher + Clone,
    H2: Hasher + Clone,
    S1: Clone + PartialEq + Debug,
    S2: Clone + PartialEq + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SeededKMBloomFilter{{seed1: {:?}, seed2: {:?}, {:?}}}",
            self.seed1, self.seed2, self.bitset
        )
    }
}

impl<H1, H2, S1, S2> BloomFilter for SeededKMBloomFilter<H1, H2, S1, S2>
where
    H1: Hasher + Clone,
    H2: Hasher + Clone,
    S1: Clone + PartialEq,
    S2: Clone + PartialEq,
{
    fn insert<T>(&mut self, data: &T)
    where
        T: Hash,
    {
        let (hash_a, hash_b) = self.generate_hashes(&data);

        for i in 0..self.number_of_hashers {
            self.bitset
                .set(Self::index(i, self.bits_per_hasher, hash_a, hash_b), true);
        }
    }

    fn contains<T>(&self, data: &T) -> bool
    where
        T: Hash,
    {
        let (hash_a, hash_b) = self.generate_hashes(data);

        for i in 0..self.number_of_hashers {
            if !self
                .bitset
                .get(Self::index(i, self.bits_per_hasher, hash_a, hash_b))
            {
                return false;
            }
        }

        return true;
    }
}

impl<H1, H2, S1, S2> BloomFilterData for SeededKMBloomFilter<H1, H2, S1, S2>
where
    H1: Hasher + Clone,
    H2: Hasher + Clone,
    S1: Clone + PartialEq,
    S2: Clone + PartialEq,
{
    type DataType = crate::bitset::Bitset;

    fn number_of_hashers(&self) -> usize {
        self.number_of_hashers
    }

    fn data(&self) -> &Self::DataType {
        &self.bitset
    }

    fn bits_per_hasher(&self) -> usize {
        self.bits_per_hasher
    }

    fn set_data(&mut self, data: Self::DataType) {
        self.bitset = data;
    }
}
