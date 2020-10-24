use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
};
mod bitset;

// capacity
// probability false positive
// number of hash functions

pub trait ResettableHasher: Hasher {
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

impl Debug for BloomFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bloomer: {:?}", self.hits)
    }
}
pub struct BloomFilter {
    hashers: Vec<Box<dyn ResettableHasher>>,
    hits: Vec<bool>,
    capacity: usize,
    count: usize,
}

impl BloomFilter {
    pub fn new(capacity: usize, hashers: Vec<Box<dyn ResettableHasher>>) -> Self {
        Self {
            hits: vec![false; capacity * hashers.len()],
            hashers,
            capacity,
            count: 0,
        }
    }
}

impl BloomFilter {
    pub fn insert<T>(&mut self, data: T)
    where
        T: Hash,
    {
        for (i, hasher) in self.hashers.iter_mut().enumerate() {
            hasher.reset();
            data.hash(hasher);
            self.hits[BloomFilter::index(i, self.capacity, hasher.finish())] = true;
        }

        self.count += 1;
    }

    pub fn check<T>(&mut self, data: &T) -> bool
    where
        T: Hash,
    {
        for (i, hasher) in self.hashers.iter_mut().enumerate() {
            hasher.reset();
            data.hash(hasher);
            if !self.hits[BloomFilter::index(i, self.capacity, hasher.finish())] {
                return false;
            }
        }
        return true;
    }

    pub fn false_positive_probability(&self) -> f64 {
        (1.0 - std::f64::consts::E
            .powf(-(self.hashers.len() as f64) * self.count as f64 / self.capacity as f64))
        .powf(self.capacity as f64)
    }

    fn index(i: usize, capacity: usize, hash: u64) -> usize {
        i * capacity + hash as usize % capacity
    }
}

impl Default for BloomFilter {
    fn default() -> Self {
        todo!()
    }
}
