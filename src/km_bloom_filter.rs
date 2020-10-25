use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use crate::{bitset::Bitset, BloomFilter};

/// Bloom filter implementation using the improvements described in
/// Kirsch A., Mitzenmacher M. (2006) Less Hashing, Same Performance: Building a Better Bloom Filter.
/// In: Azar Y., Erlebach T. (eds) Algorithms – ESA 2006. ESA 2006. Lecture Notes in Computer Science, vol 4168.
/// Springer, Berlin, Heidelberg. https://doi.org/10.1007/11841036_42
pub struct KMBloomFilter<H1, H2>
where
    H1: Hasher + Default,
    H2: Hasher + Default,
{
    hash_count: usize,
    hits: Bitset,
    bits_per_hash: usize,
    _phantom: PhantomData<(H1, H2)>,
}

impl<H1, H2> KMBloomFilter<H1, H2>
where
    H1: Hasher + Default,
    H2: Hasher + Default,
{
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
            hits: Bitset::new(bits_per_hash * hash_count),
            hash_count,
            bits_per_hash,
            _phantom: PhantomData,
        }
    }

    /// Approximate number of elements stored.
    pub fn element_count(&self) -> f64 {
        -(self.bits_per_hash as f64)
            * (1.0
                - (self.hits.count_ones() as f64) / ((self.hash_count * self.bits_per_hash) as f64))
                .ln()
    }

    /// Current approximate probability of checks returning a false positive.
    pub fn false_positive_probability(&self) -> f64 {
        (1.0 - std::f64::consts::E.powf(-self.element_count() / self.bits_per_hash as f64))
            .powf(self.hash_count as f64)
    }

    pub fn hash_count(&self) -> usize {
        self.hash_count
    }

    pub fn union(&self, other: &Self) -> Self {
        if self.hash_count != other.hash_count || self.bits_per_hash != other.bits_per_hash {
            panic!("unable to union k-m bloom filters with different configurations");
        }
        Self {
            hash_count: self.hash_count,
            hits: self.hits.union(&other.hits),
            bits_per_hash: self.bits_per_hash,
            _phantom: self._phantom,
        }
    }

    pub fn intersect(&self, other: &Self) -> Self {
        if self.hash_count != other.hash_count || self.bits_per_hash != other.bits_per_hash {
            panic!("unable to intersect k-m bloom filters with different configurations");
        }
        let na = self.element_count();
        let nb = other.element_count();
        let naub = self.union(&other).element_count();
        println!("element count: {}", na + nb - naub);
        Self {
            hash_count: self.hash_count,
            hits: self.hits.intersect(&other.hits),
            bits_per_hash: self.bits_per_hash,
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
        write!(f, "KMBloomFilter{{{:?}}}", self.hits)
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

        for i in 0..self.hash_count {
            self.hits
                .set(Self::index(i, self.bits_per_hash, hash_a, hash_b), true);
        }
    }

    fn check<T>(&self, data: &T) -> bool
    where
        T: Hash,
    {
        let (hash_a, hash_b) = self.generate_hashes(data);

        for i in 0..self.hash_count {
            if !self
                .hits
                .get(Self::index(i, self.bits_per_hash, hash_a, hash_b))
            {
                return false;
            }
        }

        return true;
    }
}
