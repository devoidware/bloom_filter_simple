use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use crate::{
    approximate_element_count, approximate_false_positive_probability, bitset::Bitset,
    optimal_bit_count, optimal_number_of_hashers, BloomFilter,
};

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
///     // We plan on storing at most 10 elements
///     let desired_capacity = 10;
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
pub struct KMBloomFilter<H1, H2>
where
    H1: Hasher + Default,
    H2: Hasher + Default,
{
    number_of_hashers: usize,
    bitset: Bitset,
    bits_per_hasher: usize,
    // Phantom data for saving which concrete Hasher types are used
    _phantom: PhantomData<(H1, H2)>,
}

impl<H1, H2> KMBloomFilter<H1, H2>
where
    H1: Hasher + Default,
    H2: Hasher + Default,
{
    /// Initialize a new instance of KMBloomFilter that guarantees that the false positive rate
    /// is less than *desired_false_positive_probability* for up to *desired_capacity*
    /// elements.
    ///
    /// KMBloomFilter uses two hash functions *H1* and *H2* to simulate an arbitrary number of hash
    /// functions. *H1* and *H2* are specified as type parameters (see examples): KMBloomFilter<H1, H2>.
    ///
    /// ***You have to use two different hash functions for *H1* and *H2*!***
    /// # Examples
    /// ```
    /// use bloom_filter_simple::{BloomFilter,KMBloomFilter};
    /// use ahash::AHasher;
    /// use std::collections::hash_map::DefaultHasher;
    ///
    /// fn main() {
    ///     // We plan on storing at most 10 elements
    ///     let desired_capacity = 10;
    ///     // We want to assure that the chance of a false positive is less than 0.0001.
    ///     let desired_fp_probability = 0.0001;
    ///
    ///     // We initialize a new KMBloomFilter by specifying the desired Hashers as type parameters
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
            _phantom: PhantomData,
        }
    }

    /// Approximate number of elements stored.
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
    pub fn approximate_current_false_positive_probability(&self) -> f64 {
        approximate_false_positive_probability(
            self.number_of_hashers,
            self.bits_per_hasher,
            self.approximate_element_count(),
        )
    }

    pub fn union(&self, other: &Self) -> Self {
        if self.number_of_hashers != other.number_of_hashers
            || self.bits_per_hasher != other.bits_per_hasher
        {
            panic!("unable to union k-m bloom filters with different configurations");
        }
        Self {
            number_of_hashers: self.number_of_hashers,
            bitset: self.bitset.union(&other.bitset),
            bits_per_hasher: self.bits_per_hasher,
            _phantom: self._phantom,
        }
    }

    pub fn intersect(&self, other: &Self) -> Self {
        if self.number_of_hashers != other.number_of_hashers
            || self.bits_per_hasher != other.bits_per_hasher
        {
            panic!("unable to intersect k-m bloom filters with different configurations");
        }
        Self {
            number_of_hashers: self.number_of_hashers,
            bitset: self.bitset.intersect(&other.bitset),
            bits_per_hasher: self.bits_per_hasher,
            _phantom: self._phantom,
        }
    }

    fn generate_hashes<T>(&self, data: &T) -> (u64, u64)
    where
        T: Hash,
    {
        let mut hasher = H1::default();
        data.hash(&mut hasher);
        let hash_a = hasher.finish();

        let mut hasher = H2::default();
        data.hash(&mut hasher);
        let hash_b = hasher.finish();

        (hash_a, hash_b)
    }

    fn index(i: usize, bits_per_hash: usize, hash_a: u64, hash_b: u64) -> usize {
        i * bits_per_hash
            + hash_a.wrapping_add((i as u64).wrapping_mul(hash_b)) as usize % bits_per_hash
    }
}

impl<H1, H2> Debug for KMBloomFilter<H1, H2>
where
    H1: Hasher + Default,
    H2: Hasher + Default,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "KMBloomFilter{{{:?}}}", self.bitset)
    }
}

impl<H1, H2> BloomFilter for KMBloomFilter<H1, H2>
where
    H1: Hasher + Default,
    H2: Hasher + Default,
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
