use crate::{
    approximate_element_count, approximate_false_positive_probability, bitset::Bitset,
    optimal_bit_count, optimal_number_of_hashers, BloomFilter,
};
use ahash::AHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

/// A bloom filter that uses a single Hasher that can be seeded to simulate an arbitrary number
/// of hash functions.
///
/// Internally, the implementation uses *ahash::AHasher*.
pub struct SeededBloomFilter {
    number_of_hashers: usize,
    bitset: Bitset,
    bits_per_hasher: usize,
}

impl SeededBloomFilter {
    /// Initialize a new instance of SeededBloomFilter that guarantees that the false positive rate
    /// is less than *desired_false_positive_probability* for up to *desired_capacity*
    /// elements.
    ///
    /// SeededBloomFilter uses a single hash function that can be seeded to simulate an arbitrary
    /// number of hash functions.
    ///
    /// # Examples
    /// ```
    /// use bloom_filter::{BloomFilter,SeededBloomFilter};
    ///
    /// fn main() {
    ///     // We plan on storing at most 10 elements
    ///     let desired_capacity = 10;
    ///     // We want to assure that the chance of a false positive is less than 0.0001.
    ///     let desired_fp_probability = 0.0001;
    ///
    ///     // We initialize a new SeededBloomFilter by specifying the desired Hashers as type
    ///     // parameters
    ///     let mut filter = SeededBloomFilter::new(desired_capacity, desired_fp_probability);
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

    fn index<T>(i: usize, bits_per_hash: usize, data: &T) -> usize
    where
        T: Hash,
    {
        let mut hasher = AHasher::new_with_keys(i as u128, i as u128);
        data.hash(&mut hasher);
        i * bits_per_hash + hasher.finish() as usize % bits_per_hash
    }
}

impl Debug for SeededBloomFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SeededBloomFilter{{{:?}}}", self.bitset)
    }
}

impl BloomFilter for SeededBloomFilter {
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
