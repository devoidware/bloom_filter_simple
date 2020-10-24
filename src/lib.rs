#![allow(dead_code)]
use std::{collections::hash_map::DefaultHasher, hash::Hash};

mod bitset;
mod km_bloom_filter;
mod seeded_bloom_filter;

pub use km_bloom_filter::KMBloomFilter;
pub use seeded_bloom_filter::SeededBloomFilter;

/**
 A default implementation of BloomFilter using ahash::AHasher and collections::hash_map::DefaultHasher.

 DefaultBloomFilter is implemented as a type definition `type DefaultBloomFilter = BloomFilter<ahash::AHasher, DefaultHasher>;`
 # Examples
 ```
 use bloom_filter::{DefaultBloomFilter,BloomFilter};

 fn simple_bloom_filter_test() {
     let desired_capacity = 1_000_000;
     let false_positive_probability = 0.0001;
     let mut bloom_filter = DefaultBloomFilter::new(desired_capacity, false_positive_probability);

     bloom_filter.insert(&"Hello!");
     bloom_filter.insert(&34);

     assert!(bloom_filter.check(&"Hello!"));
     assert!(bloom_filter.check(&34));
     assert_eq!(false, bloom_filter.check(&"Not in filter"));
 }
 ```
*/
pub type DefaultBloomFilter = KMBloomFilter<ahash::AHasher, DefaultHasher>;

pub trait BloomFilter {
    fn insert<T: Hash>(&mut self, data: &T);
    fn check<T: Hash>(&self, data: &T) -> bool;
}
