#![allow(dead_code)]
use std::{
    collections::hash_map::DefaultHasher,
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

mod bitset;
use bitset::Bitset;

pub type DefaultBloomFilter = BloomFilter<DefaultHasher, DefaultHasher>;

pub struct BloomFilter<H1, H2>
where
    H1: Hasher + Default,
    H2: Hasher + Default,
{
    hash_count: usize,
    hits: Bitset,
    bits_per_hash: usize,
    element_count: usize,
    _phantom: PhantomData<(H1, H2)>,
}

impl<H1, H2> Debug for BloomFilter<H1, H2>
where
    H1: Hasher + Default,
    H2: Hasher + Default,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BloomFilter{{{:?}}}", self.hits)
    }
}

impl<H1, H2> BloomFilter<H1, H2>
where
    H1: Hasher + Default,
    H2: Hasher + Default,
{
    pub fn new(desired_capacity: usize, false_positive_probability: f64) -> Self {
        // using formulas to calculate optimum size and hash function count
        // m = ceil((n * ln(p)) / ln(1 / pow(2, ln(2)))); ln (1/(2^ln(2))) is approx. -0.48045301391
        // k = round((m / n) * ln(2)); ln(2) is approx. 0.693147
        let bit_count = ((desired_capacity as f64 * false_positive_probability.ln())
            / (1.0 / 2.0f64.powf(2.0f64.ln())).ln())
        .ceil();
        let hash_count =
            ((bit_count as f64 / desired_capacity as f64) * 2.0f64.ln()).round() as usize;
        let bits_per_hash = (bit_count / hash_count as f64).ceil() as usize;
        Self {
            hits: Bitset::new(bits_per_hash * hash_count),
            hash_count,
            bits_per_hash,
            element_count: 0,
            _phantom: PhantomData,
        }
    }

    pub fn insert<T>(&mut self, data: T)
    where
        T: Hash,
    {
        let (hash_a, hash_b) = self.generate_hashes(&data);

        for i in 0..self.hash_count {
            self.hits
                .set(Self::index(i, self.bits_per_hash, hash_a, hash_b), true);
        }

        self.element_count += 1;
    }

    pub fn check<T>(&self, data: &T) -> bool
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

    pub fn false_positive_probability(&self) -> f64 {
        (1.0 - std::f64::consts::E.powf(-(self.element_count as f64) / self.bits_per_hash as f64))
            .powf(self.hash_count as f64)
    }

    pub fn hash_count(&self) -> usize {
        self.hash_count
    }

    fn generate_hashes<T>(&self, data: &T) -> (u64, u64)
    where
        T: Hash,
    {
        let mut hasher = H1::default();
        data.hash(&mut hasher);
        let hash_a = hasher.finish();

        let mut hasher = H2::default();
        hash_a.hash(&mut hasher);
        let hash_b = hasher.finish();

        (hash_a, hash_b)
    }

    fn index(i: usize, bits_per_hash: usize, hash_a: u64, hash_b: u64) -> usize {
        i * bits_per_hash
            + hash_a.wrapping_add((i as u64).wrapping_mul(hash_b)) as usize % bits_per_hash
    }
}
