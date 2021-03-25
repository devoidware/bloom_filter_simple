use crate::{
    approximate_element_count, approximate_false_positive_probability, bitset::Bitset,
    optimal_bit_count, optimal_number_of_hashers, BloomFilter, BloomFilterData,
};
use ahash::AHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

/// A bloom filter that uses a single Hasher that can be seeded to simulate an arbitrary number
/// of hash functions.
///
/// Internally, the implementation uses *ahash::AHasher*.
#[derive(Clone)]
pub struct SingleHasherBloomFilter {
    number_of_hashers: usize,
    bitset: Bitset,
    bits_per_hasher: usize,
}

impl SingleHasherBloomFilter {
    /// Initialize a new instance of SingleHasherBloomFilter that guarantees that the false positive rate
    /// is less than *desired_false_positive_probability* for up to *desired_capacity*
    /// elements.
    ///
    /// SingleHasherBloomFilter uses a single hash function that can be seeded to simulate an arbitrary
    /// number of hash functions.
    ///
    /// # Panics
    ///
    /// Panics if desired_capacity == 0
    ///
    /// # Examples
    /// ```
    /// use bloom_filter_simple::{BloomFilter,SingleHasherBloomFilter};
    ///
    /// fn main() {
    ///     // We plan on storing at most 10 elements
    ///     let desired_capacity = 10;
    ///     // We want to assure that the chance of a false positive is less than 0.0001.
    ///     let desired_fp_probability = 0.0001;
    ///
    ///     // We initialize a new SingleHasherBloomFilter by specifying the desired Hashers as type
    ///     // parameters
    ///     let mut filter = SingleHasherBloomFilter::new(desired_capacity, desired_fp_probability);
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
    /// use bloom_filter_simple::{BloomFilter,SingleHasherBloomFilter};
    ///
    /// fn main() {
    ///     // The configuration of both bloom filters has to be the same
    ///     let desired_capacity = 10_000;
    ///     let desired_fp_probability = 0.0001;
    ///
    ///     // We initialize two new SingleHasherBloomFilter
    ///     let mut filter_one = SingleHasherBloomFilter::new(desired_capacity, desired_fp_probability);
    ///     let mut filter_two = SingleHasherBloomFilter::new(desired_capacity, desired_fp_probability);
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
        }
    }

    /// Checks whether two bloom filters were created with the same desired capacity and desired false
    /// positive probability.
    pub fn eq_configuration(&self, other: &Self) -> bool {
        self.number_of_hashers == other.number_of_hashers
            && self.bits_per_hasher == other.bits_per_hasher
    }

    fn index<T>(i: usize, bits_per_hash: usize, data: &T) -> usize
    where
        T: Hash,
    {
        let mut hasher = AHasher::new_with_keys(i as u128, i as u128);
        data.hash(&mut hasher);
        i * bits_per_hash + hasher.finish() as usize % bits_per_hash
    }
}

impl Debug for SingleHasherBloomFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SingleHasherBloomFilter{{{:?}}}", self.bitset)
    }
}

impl BloomFilter for SingleHasherBloomFilter {
    fn insert<T>(&mut self, data: &T)
    where
        T: Hash,
    {
        for i in 0..self.number_of_hashers {
            self.bitset
                .set(Self::index(i, self.bits_per_hasher, &data), true);
        }
    }

    fn contains<T>(&self, data: &T) -> bool
    where
        T: Hash,
    {
        for i in 0..self.number_of_hashers {
            if !self.bitset.get(Self::index(i, self.bits_per_hasher, &data)) {
                return false;
            }
        }

        return true;
    }
}

impl BloomFilterData for SingleHasherBloomFilter {
    type DataType = crate::bitset::Bitset;

    fn number_of_hashers(&self) -> usize {
        self.number_of_hashers
    }

    fn bits_per_hasher(&self) -> usize {
        self.bits_per_hasher
    }

    fn data(&self) -> &Self::DataType {
        &self.bitset
    }

    fn set_data(&mut self, data: Self::DataType) {
        self.bitset = data;
    }
}
