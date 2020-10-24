#![allow(dead_code)]
use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
};

mod bitset;
use bitset::Bitset;

pub trait ResettableHasher: Hasher + Default {
    fn reset(&mut self);
}

impl<T> ResettableHasher for T
where
    T: Default + Hasher,
{
    fn reset(&mut self) {
        *self = Self::default();
    }
}

pub struct BloomFilter<H>
where
    H: ResettableHasher,
{
    hasher: Box<H>,
    hash_count: usize,
    hits: Bitset,
    capacity: usize,
    element_count: usize,
}

impl<H> Debug for BloomFilter<H>
where
    H: ResettableHasher,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BloomFilter{{{:?}}}", self.hits)
    }
}

impl<H> BloomFilter<H>
where
    H: ResettableHasher,
{
    pub fn new(capacity: usize, false_positive_probability: f64) -> Self {
        // using formulas to calculate optimum size and hash function count
        // m = ceil((n * ln(p)) / ln(1 / pow(2, ln(2))));
        // k = round((m / n) * ln(2));
        let bit_count =
            ((capacity as f64 * false_positive_probability.ln()) / -0.48045301391).ceil(); // ln (1/(2^ln(2))) is approx. -0.48045301391
        let hash_count = ((bit_count as f64 / capacity as f64) * 0.693147).round() as usize; // ln(2) is approx. 0.693147
        let bits_per_hash = (bit_count / hash_count as f64).ceil() as usize;
        Self {
            hits: Bitset::new(bits_per_hash * hash_count),
            hasher: Box::new(H::default()),
            hash_count,
            capacity: bits_per_hash,
            element_count: 0,
        }
    }

    pub fn insert<T>(&mut self, data: T)
    where
        T: Hash,
    {
        let (hash_a, hash_b) = self.generate_hashes(&data);

        for i in 0..self.hash_count {
            self.hits
                .set(Self::index(i, self.capacity, hash_a, hash_b), true);
        }

        self.element_count += 1;
    }

    pub fn check<T>(&mut self, data: &T) -> bool
    where
        T: Hash,
    {
        let (hash_a, hash_b) = self.generate_hashes(data);

        for i in 0..self.hash_count {
            if !self.hits.get(Self::index(i, self.capacity, hash_a, hash_b)) {
                return false;
            }
        }

        return true;
    }

    pub fn false_positive_probability(&self) -> f64 {
        (1.0 - std::f64::consts::E.powf(-(self.element_count as f64) / self.capacity as f64))
            .powf(self.hash_count as f64)
    }

    fn generate_hashes<T>(&mut self, data: &T) -> (u64, u64)
    where
        T: Hash,
    {
        data.hash(&mut self.hasher);
        let hash_a = self.hasher.finish();
        self.hasher.reset();

        hash_a.hash(&mut self.hasher);
        let hash_b = self.hasher.finish();
        self.hasher.reset();

        (hash_a, hash_b)
    }

    fn index(i: usize, capacity: usize, hash_a: u64, hash_b: u64) -> usize {
        i * capacity + (hash_a.wrapping_add(i as u64)).wrapping_mul(hash_b) as usize % capacity
    }
}
