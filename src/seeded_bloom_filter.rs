use crate::{bitset::Bitset, BloomFilter};
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
    element_count: usize,
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
        // using formulas to calculate optimum size and hash function count
        // m = ceil((n * ln(p)) / ln(1 / pow(2, ln(2)))); ln (1/(2^ln(2))) is approx. -0.48045301391
        // k = round((m / n) * ln(2)); ln(2) is approx. 0.693147
        let bit_count = ((desired_capacity as f64 * desired_false_positive_probability.ln())
            / (1.0 / 2.0f64.powf(2.0f64.ln())).ln())
        .ceil();
        let hash_count =
            ((bit_count as f64 / desired_capacity as f64) * 2.0f64.ln()).round() as usize;
        let bits_per_hash = (bit_count / hash_count as f64).ceil() as usize;
        Self {
            bitset: Bitset::new(bits_per_hash * hash_count),
            number_of_hashers: hash_count,
            bits_per_hasher: bits_per_hash,
            element_count: 0,
        }
    }

    /// Return the current false positive probability which depends on the current number of elements
    /// in the filter.
    pub fn false_positive_probability(&self) -> f64 {
        (1.0 - std::f64::consts::E.powf(-(self.element_count as f64) / self.bits_per_hasher as f64))
            .powf(self.number_of_hashers as f64)
    }

    /// Return the number of hash functions that are simulated by this instance.
    pub fn hash_count(&self) -> usize {
        self.number_of_hashers
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

        self.element_count += 1;
    }

    fn check<T>(&self, data: &T) -> bool
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
