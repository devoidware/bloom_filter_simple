use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use crate::{bitset::Bitset, BloomFilter};

/// Bloom filter implementation using the improvements described Kirsch and Mitzenmacher:
///
/// > Kirsch A., Mitzenmacher M. (2006) Less Hashing, Same Performance: Building a Better Bloom Filter.
/// In: Azar Y., Erlebach T. (eds) Algorithms – ESA 2006. ESA 2006. Lecture Notes in Computer Science, vol 4168.
/// Springer, Berlin, Heidelberg. https://doi.org/10.1007/11841036_42
///
/// # Examples
/// ```
/// use bloom_filter::{BloomFilter,KMBloomFilter};
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
///     // You can check whether a value has been inserted into by the filter before.
///     assert_eq!(false, filter.check(&3));
///     assert_eq!(true, filter.check(&5));
///     assert_eq!(true, filter.check(&"Some text"));
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
    /// use bloom_filter::{BloomFilter,KMBloomFilter};
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
            _phantom: PhantomData,
        }
    }

    /// Approximate number of elements stored.
    pub fn element_count(&self) -> f64 {
        -(self.bits_per_hasher as f64)
            * (1.0
                - (self.bitset.count_ones() as f64) / ((self.number_of_hashers * self.bits_per_hasher) as f64))
                .ln()
    }

    /// Return the current approximate false positive probability which depends on the current number of elements
    /// in the filter.
    pub fn false_positive_probability(&self) -> f64 {
        (1.0 - std::f64::consts::E.powf(-self.element_count() / self.bits_per_hasher as f64))
            .powf(self.number_of_hashers as f64)
    }

    /// Return the number of hash functions that are simulated by this instance.
    pub fn hash_count(&self) -> usize {
        self.number_of_hashers
    }

    pub fn union(&self, other: &Self) -> Self {
        if self.number_of_hashers != other.number_of_hashers || self.bits_per_hasher != other.bits_per_hasher {
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
        if self.number_of_hashers != other.number_of_hashers || self.bits_per_hasher != other.bits_per_hasher {
            panic!("unable to intersect k-m bloom filters with different configurations");
        }
        let na = self.element_count();
        let nb = other.element_count();
        let naub = self.union(&other).element_count();
        println!("element count: {}", na + nb - naub);
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

    fn check<T>(&self, data: &T) -> bool
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
