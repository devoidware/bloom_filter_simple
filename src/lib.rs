//! bloom-filter-simple is a library that offers different implementations of a data
//! structure for filtering elements. The data structure is based on the ideas presented by Burton
//! Howard Bloom and is therefore known as bloom filter:
//! > Burton H. Bloom. 1970. Space/time trade-offs in hash coding with allowable errors. Commun.
//! ACM 13, 7 (July 1970), 422–426. DOI: [https://doi.org/10.1145/362686.362692](https://doi.org/10.1145/362686.362692)
//!
//! # Overview
//! Basic description taken from [Wikipedia](https://en.wikipedia.org/wiki/Bloom_filter):
//!
//! > "A Bloom filter is a space-efficient probabilistic data structure, conceived by Burton Howard
//! Bloom in 1970, that is used to test whether an element is a member of a set. False positive
//! matches are possible, but false negatives are not – in other words, a query returns either
//! "possibly in set" or "definitely not in set". Elements can be added to the set, but not removed
//! (though this can be addressed with the counting Bloom filter variant); the more items added, the
//! larger the probability of false positives."
//!
//! > ("Bloom filter". Definition, para. 1. In Wikipedia. Retrieved December 02, 2020, from https://en.wikipedia.org/wiki/Bloom_filter)
//!
//! # Bloom Filter Implementations
//! The library offers two basic types of bloom filter implementations.
//!
//! ## Kirsch-Mitzenmacher Bloom Filter (KMBloomFilter)
//! This type of bloom filter uses two hashers to simulate an arbitrary number of additional hash functions.
//!
//! The implementation is based on the work of [Kirsch and Mitzenmacher](https://doi.org/10.1007/11841036_42) \[1\].
//! In their work, they demonstrated that it is possible to apply simulated hash functions in a bloom
//! filter effectively, i.e., without loss in the asymptotic false positive probability.
//! Given two hash functions *h_1(x)* and *h_2(x)*, an *i*-th additional hash function *g_i(x)* can be
//! simulated as *g_i(x) = h_1(x) + i* \* *h_2(x)*.
//!
//!  > [1] Kirsch A., Mitzenmacher M. (2006) Less Hashing, Same Performance: Building a Better Bloom Filter.
//! In: Azar Y., Erlebach T. (eds) Algorithms – ESA 2006. ESA 2006. Lecture Notes in Computer Science, vol 4168.
//! Springer, Berlin, Heidelberg. https://doi.org/10.1007/11841036_42
//!
//! ## Seeded Bloom Filter (SeededBloomFilter)
//! A bloom filter that uses a single Hasher that can be seeded to simulate an arbitrary number of hash functions.
//! Internally, the implementation uses [ahash::AHasher](https://crates.io/crates/ahash).
//!
//! # Examples
//! In the following, you can find simple examples of how to initialize and use the different bloom filter types.
//!
//! ## Default Bloom Filter
//! The crate offers a default type for a KMBloomFilter that uses *ahash::AHasher* and Rust's
//! *std::collections::hash_map::DefaultHasher* to simulate more hash functions. We compared
//! different hash functions for use by KMBloomFilter, and this combination yielded the best results
//! with respect to the filter's false positive probability.
//!
//! We recommend using DefaultBloomFilter for quickly getting started.
//! ```
//! use bloom_filter_simple::{BloomFilter,DefaultBloomFilter};
//!
//! fn main() {
//!     // We plan on storing at most 10,000 elements
//!     let desired_capacity = 10_000;
//!     // The chance of a false positive increases with each inserted element.
//!     // This parameter specifies that the chance should be less than 0.01% (0.0001)
//!     // when the desired capacity has been reached. In other words, the chance
//!     // that the bloom filter returns true when checking whether a novel element
//!     // has been inserted before is less than 0.01% (0.0001).
//!     let desired_fp_probability = 0.0001;
//!
//!     let mut filter = DefaultBloomFilter::new(desired_capacity, desired_fp_probability);
//!
//!     // You can insert any type implementing the Hash trait. The bloom filter does
//!     // not store the inserted elements but only their hashes. Hence, there is no
//!     // transfer of ownership required.
//!     filter.insert(&5i32);
//!     filter.insert(&"Some text");
//!     filter.insert(&10_000usize);
//!
//!     // You can check whether a value has been inserted into the filter before.
//!     assert_eq!(false, filter.contains(&3));
//!     assert_eq!(true, filter.contains(&5));
//!     assert_eq!(true, filter.contains(&"Some text"));
//! }
//! ```
//!
//! ## KMBloomFilter
//! Initialization and application of a KMBloomFilter.
//! ```
//! use bloom_filter_simple::{BloomFilter,KMBloomFilter};
//! use ahash::AHasher;
//! use std::collections::hash_map::DefaultHasher;
//!
//! fn main() {
//!     // We plan on storing at most 10,000 elements
//!     let desired_capacity = 10_000;
//!     // We want to assure that the chance of a false positive is less than 0.01% (0.0001)
//!     // for up to desired_capacity elements.
//!     let desired_fp_probability = 0.0001;
//!
//!     // We initialize a new KMBloomFilter by specifying the desired Hashers as type
//!     // parameters. It is possible to use any type that implements Hasher + Default.
//!     // Default is required to receive a new instance of a hasher after a value was
//!     // hashed, because the Hasher trait does not provide an interface for resetting
//!     // a hasher implementing it. This is required to receive the same hash value
//!     // when inserting or checking the same element multiple times.
//!     let mut filter: KMBloomFilter<AHasher, DefaultHasher> = KMBloomFilter::new(
//!         desired_capacity,
//!         desired_fp_probability
//!     );
//!
//!     // You can insert any type implementing the Hash trait. The bloom filter does not
//!     // store the inserted elements but only their hashes. Hence, there is no transfer
//!     // of ownership required.
//!     filter.insert(&5i32);
//!     filter.insert(&"Some text");
//!     filter.insert(&10_000usize);
//!
//!     // You can check whether a value has been inserted into the filter before.
//!     assert_eq!(false, filter.contains(&3));
//!     assert_eq!(true, filter.contains(&5));
//!     assert_eq!(true, filter.contains(&"Some text"));
//! }
//! ```
//!
//! ## SeededBloomFilter
//! Initialization and application of a SeededBloomFilter.
//! ```
//! use bloom_filter_simple::{BloomFilter,SeededBloomFilter};
//!
//! fn main() {
//!     // We plan on storing at most 10,000 elements
//!     let desired_capacity = 10_000;
//!     // We want to assure that the chance of a false positive is less than 0.0001
//!     // for up to desired_capacity elements.
//!     let desired_fp_probability = 0.0001;
//!
//!     // A SeededBloomFilter uses a single seeded ahash::AHasher internally.
//!     let mut filter = SeededBloomFilter::new(desired_capacity, desired_fp_probability);
//!
//!     // You can insert any type implementing the Hash trait. The bloom filter does
//!     // not store the inserted elements but only their hashes. Hence, there is no
//!     // transfer of ownership required.
//!     filter.insert(&5i32);
//!     filter.insert(&"Some text");
//!     filter.insert(&10_000usize);
//!
//!     // You can check whether a value has been inserted into the filter before.
//!     assert_eq!(false, filter.contains(&3));
//!     assert_eq!(true, filter.contains(&5));
//!     assert_eq!(true, filter.contains(&"Some text"));
//! }
//! ```

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
 use bloom_filter_simple::{DefaultBloomFilter,BloomFilter};

 fn simple_bloom_filter_test() {
     let desired_capacity = 1_000_000;
     let false_positive_probability = 0.0001;
     let mut bloom_filter = DefaultBloomFilter::new(desired_capacity, false_positive_probability);

     bloom_filter.insert(&"Hello!");
     bloom_filter.insert(&34);

     assert!(bloom_filter.contains(&"Hello!"));
     assert!(bloom_filter.contains(&34));
     assert_eq!(false, bloom_filter.contains(&"Not in filter"));
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
    /// How *insert* of a type implementing BloomFilter might be used:
    /// ```
    /// use bloom_filter_simple::{BloomFilter, DefaultBloomFilter};
    ///
    /// fn bloom_filter_insert() {
    ///     let mut bloom_filter = DefaultBloomFilter::new(5, 0.001);
    ///     bloom_filter.insert(&"Hello!");
    ///     bloom_filter.insert(&5);
    ///     bloom_filter.insert(&"Hello!");
    ///
    ///     assert_eq!(true, bloom_filter.contains(&"Hello!"));
    /// }
    /// ```
    fn insert<T: Hash>(&mut self, data: &T);

    /// Check whether data is contained in the bloom filter.
    ///
    /// # Intended Behavior
    /// Checking whether data is contained in a bloom filter must never result in a false negative,
    /// i.e., if an element 'x' has been inserted into the filter, contains(&x) will *always* return true.
    ///
    /// In contrast, contains can result in false positive, i.e., contains(&x) can return true, even if
    /// x has not been inserted yet. The chance of this happending depends on the number of elements
    /// in the bloom filter, and the number of hash functions that are used. When initializing one
    /// of the filters provided in this crate, you can specify the desired false positive probability.
    ///
    /// A type implementing BloomFilter should implement *contains* with respect to the following points:
    /// * *contains(&x)* **must** return *true* if *x* has been inserted into the filter
    /// * *contains(&x)* **can** return *true* even if *x* has **not** been inserted into the filter
    /// * It should be possible to check any type implementing Hash.
    ///
    /// # Examples
    /// How contains of a type implementing BloomFilter might be used:
    /// ```
    /// use bloom_filter_simple::{BloomFilter, DefaultBloomFilter};
    /// fn bloom_filter_insert() {
    ///     let mut bloom_filter = DefaultBloomFilter::new(5, 0.001);
    ///     bloom_filter.insert(&"Hello!");
    ///     // This assert will never fail
    ///     assert_eq!(true, bloom_filter.contains(&"Hello!"));
    ///     // This assert can fail with a probability of p(fp) < 0.001
    ///     assert_eq!(false, bloom_filter.contains(&"Goodbye!"));
    /// }
    /// ```
    fn contains<T: Hash>(&self, data: &T) -> bool;
}

/// Calculate the optimal bit count to satisfy the desired constraints.
/// Formula taken from Sagi Kedmi:
/// > S. Kedmi, ["Bloom Filters for the Perplexed"](https://sagi.io/bloom-filters-for-the-perplexed/), July 2017 [Accessed: 02.12.2020]
fn optimal_bit_count(desired_capacity: usize, desired_false_positive_probability: f64) -> usize {
    (-(desired_capacity as f64 * desired_false_positive_probability.ln())
        / (2.0f64.ln().powi(2)))
    .ceil() as usize
}

/// Calculate the optimal number of hashers to satisfy the desired constraints.
/// Formula taken from Sagi Kedmi:
/// > S. Kedmi, ["Bloom Filters for the Perplexed"](https://sagi.io/bloom-filters-for-the-perplexed/), July 2017 [Accessed: 02.12.2020]
fn optimal_number_of_hashers(desired_capacity: usize, bit_count: usize) -> usize {
    ((bit_count as f64 / desired_capacity as f64) * 2.0f64.ln()).round() as usize
}

/// Approximate number of elements stored.
/// Formula taken from Wikipedia:
/// > Wikipedia, ["Bloom filter"](https://en.wikipedia.org/wiki/Bloom_filter#Approximating_the_number_of_items_in_a_Bloom_filter) [Accessed: 02.12.2020]
fn approximate_element_count(
    number_of_hashers: usize,
    bits_per_hasher: usize,
    number_of_ones: usize,
) -> f64 {
    -(bits_per_hasher as f64)
        * (1.0 - (number_of_ones as f64) / ((number_of_hashers * bits_per_hasher) as f64)).ln()
}

/// Return the current approximate false positive probability which depends on the current
/// number of elements in the filter.
/// Formula taken from Sagi Kedmi:
/// > S. Kedmi, ["Bloom Filters for the Perplexed"](https://sagi.io/bloom-filters-for-the-perplexed/), July 2017 [Accessed: 02.12.2020]
fn approximate_false_positive_probability(
    number_of_hashers: usize,
    bits_per_hasher: usize,
    element_count: f64,
) -> f64 {
    (1.0 - std::f64::consts::E.powf(-element_count / bits_per_hasher as f64))
        .powf(number_of_hashers as f64)
}
