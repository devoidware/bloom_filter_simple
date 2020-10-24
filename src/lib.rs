#![allow(dead_code)]
use std::{
    collections::hash_map::DefaultHasher,
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
    hasher: Box<dyn ResettableHasher>,
    hash_count: usize,
    hits: Vec<bool>,
    capacity: usize,
    element_count: usize,
}

impl BloomFilter {
    pub fn new(capacity: usize, hash_count: usize) -> Self {
        Self {
            hits: vec![false; capacity * hash_count],
            hasher: Box::new(DefaultHasher::default()),
            hash_count,
            capacity,
            element_count: 0,
        }
    }
}

impl BloomFilter {
    pub fn insert<T>(&mut self, data: T)
    where
        T: Hash,
    {
        let (hash_a, hash_b) = self.generate_hashes(&data);

        for i in 0..self.hash_count {
            self.hits[Self::index(i, self.capacity, hash_a, hash_b)] = true;
        }

        self.element_count += 1;
    }

    pub fn check<T>(&mut self, data: &T) -> bool
    where
        T: Hash,
    {
        let (hash_a, hash_b) = self.generate_hashes(data);

        for i in 0..self.hash_count {
            if !self.hits[Self::index(i, self.capacity, hash_a, hash_b)] {
                return false;
            }
        }

        return true;
    }

    pub fn false_positive_probability(&self) -> f64 {
        (1.0 - std::f64::consts::E
            .powf(-(self.hash_count as f64) * self.element_count as f64 / self.capacity as f64))
        .powf(self.capacity as f64)
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

impl Default for BloomFilter {
    fn default() -> Self {
        todo!()
    }
}
