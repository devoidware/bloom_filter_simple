#![allow(dead_code)]
use std::collections::hash_map::DefaultHasher;

mod bitset;
mod km_bloom_filter;
mod seeded_bloom_filter;

pub use km_bloom_filter::KMBloomFilter;
pub use seeded_bloom_filter::SeededBloomFilter;

pub type DefaultBloomFilter = KMBloomFilter<ahash::AHasher, DefaultHasher>;
