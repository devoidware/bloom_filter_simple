use crate::bitset::Bitset;
use ahash::AHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub struct SeededBloomFilter {
    hash_count: usize,
    hits: Bitset,
    bits_per_hash: usize,
    element_count: usize,
}

impl Debug for SeededBloomFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SeededBloomFilter{{{:?}}}", self.hits)
    }
}

impl SeededBloomFilter {
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
        }
    }

    pub fn insert<T>(&mut self, data: T)
    where
        T: Hash,
    {
        for i in 0..self.hash_count {
            self.hits
                .set(Self::index(i, self.bits_per_hash, &data), true);
        }

        self.element_count += 1;
    }

    pub fn check<T>(&self, data: &T) -> bool
    where
        T: Hash,
    {
        for i in 0..self.hash_count {
            if !self.hits.get(Self::index(i, self.bits_per_hash, &data)) {
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

    fn index<T>(i: usize, bits_per_hash: usize, data: &T) -> usize
    where
        T: Hash,
    {
        let mut hasher = AHasher::new_with_keys(i as u128, i as u128);
        data.hash(&mut hasher);
        i * bits_per_hash + hasher.finish() as usize % bits_per_hash
    }
}
