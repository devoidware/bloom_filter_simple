#![allow(dead_code)]
use std::{collections::hash_map::DefaultHasher, hash::Hash};

mod bitset;
mod km_bloom_filter;
mod seeded_bloom_filter;

pub use km_bloom_filter::KMBloomFilter;
pub use seeded_bloom_filter::SeededBloomFilter;

pub type DefaultBloomFilter = KMBloomFilter<ahash::AHasher, DefaultHasher>;

pub trait BloomFilter {
    fn insert<T: Hash>(&mut self, data: &T);
    fn check<T: Hash>(&self, data: &T) -> bool;
}
