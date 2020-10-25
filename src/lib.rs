//! bloom-filter is a library that offers different implementations of a simple bloom filter based
//! on the initial ideas of presented by Burton Howard Bloom TODOCite.
//!
//! # Overview
//! Basic description from [Wikipedia](https://en.wikipedia.org/wiki/Bloom_filter):
//!
//! > A Bloom filter is a space-efficient probabilistic data structure, conceived by Burton Howard
//! Bloom in 1970, that is used to test whether an element is a member of a set. False positive
//! matches are possible, but false negatives are not – in other words, a query returns either
//! "possibly in set" or "definitely not in set". Elements can be added to the set, but not removed
//! (though this can be addressed with the counting Bloom filter variant); the more items added, the
//! larger the probability of false positives.
//!
//! # Features
//! ## Kirsch-Mitzenmacher Bloom Filter (KMBloomFilter)
//! This type of bloom filter uses two hashers to simulate an arbitrary number of additional hash functions.
//!
//! The implementation is based on a publication by [Kirsch and Mitzenmacher](https://doi.org/10.1007/11841036_42).
//! In their work, they demonstrated that it is possible to apply simulated hash functions in a bloom
//! filter effectively, i.e., without loss in the asymptotic false positive probability.
//!
//! Given two hash functions *h_1(x)* and *h_2(x)*, an *i*-th additional hash function *g_i(x)* can be
//! simulated as *g_i(x) = h_1(x) + i* \* *h_2(x)*.
//!
//! ## Seeded Bloom Filter (SeededBloomFilter)
//!
//!
//! # Examples
//! ## Default Bloom Filter
//! ```
//! use bloom_filter::{BloomFilter,DefaultBloomFilter};
//!
//! fn main() {
//!     // We plan on storing at most 10 elements
//!     let desired_capacity = 10;
//!     // The chance of a false positive increases with each inserted element. This parameter
//!     // specifies that it should be less than 0.0001 when the desired capacity has
//!     // been reached.
//!     // In other words, the chance that the bloom filter returns *true* when checking whether a
//!     // **novel** element has been inserted before is less than 0.0001.
//!     let desired_fp_probability = 0.0001;
//!
//!     // The crate offers a type definition for a default KMBloomFilter that applies 'AHasher' from
//!     // the 'ahash' crate, and Rust's default hasher. When testing different hash functions,
//!     // this combinations achieved the best results with respect to filter's false positive probability.
//!     let mut filter = DefaultBloomFilter::new(desired_capacity, desired_fp_probability);
//!
//!     // You can insert any type implementing the Hash trait. The bloom filter does not store the
//!     // inserted elements but only their hashes. Hence, there is no transfer of ownership required.
//!     filter.insert(&5i32);
//!     filter.insert(&"Some text");
//!     filter.insert(&10_000usize);
//!
//!     // You can check whether a value has been inserted into by the filter before.
//!     assert_eq!(false, filter.check(&3));
//!     assert_eq!(true, filter.check(&5));
//!     assert_eq!(true, filter.check(&"Some text"));
//! }
//! ```
//!
//! ## KMBloomFilter
//! ```
//! use bloom_filter::{BloomFilter,KMBloomFilter};
//! use ahash::AHasher;
//! use std::collections::hash_map::DefaultHasher;
//!
//! fn main() {
//!     // We plan on storing at most 10 elements
//!     let desired_capacity = 10;
//!     // We want to assure that the chance of a false positive is less than 0.0001 for up to
//!     // desired_capacity elements.
//!     let desired_fp_probability = 0.0001;
//!
//!     // We initialize a new KMBloomFilter by specifying the desired Hashers as type parameters.
//!     // It is possible to use any type that implements Hasher + Default.
//!     // Default is required to receive a new instance of a hasher after a value was hashed, because
//!     // the Hasher trait does not provide an interface for resetting a hasher implementing it.
//!     // This is required to receive the same hash value when inserting or checking the same element
//!     // multiple times.
//!     let mut filter: KMBloomFilter<AHasher, DefaultHasher> = KMBloomFilter::new(desired_capacity, desired_fp_probability);
//!
//!     // You can insert any type implementing the Hash trait. The bloom filter does not store the
//!     // inserted elements but only their hashes. Hence, there is no transfer of ownership required.
//!     filter.insert(&5i32);
//!     filter.insert(&"Some text");
//!     filter.insert(&10_000usize);
//!
//!     // You can check whether a value has been inserted into by the filter before.
//!     assert_eq!(false, filter.check(&3));
//!     assert_eq!(true, filter.check(&5));
//!     assert_eq!(true, filter.check(&"Some text"));
//! }
//! ```

#![allow(dead_code)]
use std::{collections::hash_map::DefaultHasher, hash::Hash};

mod bitset;
mod km_bloom_filter;
mod seeded_bloom_filter;

pub use km_bloom_filter::KMBloomFilter;
pub use seeded_bloom_filter::SeededBloomFilter;

/**
 A default implementation of KMBloomFilter using ahash::AHasher and collections::hash_map::DefaultHasher.

 DefaultBloomFilter is implemented as a type definition `type DefaultBloomFilter = KMBloomFilter<ahash::AHasher, DefaultHasher>;`
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

/// This trait defines the basic functionality supported by the bloom filters in this library.
///
pub trait BloomFilter {
    /// Insert data into the filter.
    ///
    /// # Intended Behavior
    /// A type implementing BloomFilter should implement *insert* with respect to the following points:
    /// * It should be possible to insert the same element multiple times.
    /// * It should be possible to insert any type implementing Hash.
    ///
    /// # Examples
    /// ## How a BloomFilter might be used:
    /// ```
    /// use bloom_filter::{BloomFilter, DefaultBloomFilter};
    ///
    /// fn bloom_filter_insert() {
    ///     let mut bloom_filter = DefaultBloomFilter::new(5, 0.001);
    ///     bloom_filter.insert(&"Hello!");
    ///     bloom_filter.insert(&5);
    ///     bloom_filter.insert(&"Hello!");
    ///
    ///     assert_eq!(true, bloom_filter.check(&"Hello!"));
    /// }
    /// ```
    fn insert<T: Hash>(&mut self, data: &T);

    /// Check whether data is contained in the bloom filter.
    ///
    /// # Intended Behavior
    /// Checking whether data is contained in a bloom filter must never result in a false negative,
    /// i.e., if an element 'x' has been inserted into the filter, check(&x) will *always* return true.
    ///
    /// In contrast, a check can result in false positive, i.e., check(&x) can return true, even if
    /// x has not been inserted yet. The chance of this happending depends on the number of elements
    /// in the bloom filter, and the number of hash functions that are used. When initializing one
    /// of the filters provided in this crate, you can specify the desired false positive probability.
    ///
    /// A type implementing BloomFilter should implement *check* with respect to the following points:
    /// * *check(&x)* **must** return *true* if *x* has been inserted into the filter
    /// * *check(&x)* **can** return *true* even if *x* has **not** been inserted into the filter
    /// * It should be possible to check any type implementing Hash.
    ///
    /// # Examples
    /// ## How a BloomFilter might be used
    /// ```
    /// use bloom_filter::{BloomFilter, DefaultBloomFilter};
    /// fn bloom_filter_insert() {
    ///     let mut bloom_filter = DefaultBloomFilter::new(5, 0.001);
    ///     bloom_filter.insert(&"Hello!");
    ///     // This assert will *never* fail
    ///     assert_eq!(true, bloom_filter.check(&"Hello!"));
    ///     // This assert can fail with a probability of p(fp) < 0.001
    ///     assert_eq!(false, bloom_filter.check(&"Goodbye!"));
    /// }
    /// ```
    fn check<T: Hash>(&self, data: &T) -> bool;
}
