# Overview

`bloom_filter_simple` is a library that offers different implementations of a simple bloom filter based
on the initial ideas presented by Burton Howard Bloom:
> Burton H. Bloom. 1970. Space/time trade-offs in hash coding with allowable errors. Commun.
ACM 13, 7 (July 1970), 422–426. DOI:https://doi.org/10.1145/362686.362692

Basic description from [Wikipedia](https://en.wikipedia.org/wiki/Bloom_filter):

> A Bloom filter is a space-efficient probabilistic data structure, conceived by Burton Howard
Bloom in 1970, that is used to test whether an element is a member of a set. False positive
matches are possible, but false negatives are not – in other words, a query returns either
"possibly in set" or "definitely not in set". Elements can be added to the set, but not removed
(though this can be addressed with the counting Bloom filter variant); the more items added, the
larger the probability of false positives.

# Bloom Filter Implementations

The library offers two basic types of bloom filter implementations: `KMBloomFilter` and `SeededBloomFilter`. The former uses two hash functions while the latter uses only one hash function but different seeds to generate different hashes.

# Examples

## `DefaultBloomFilter`

The `DefaultBloomFilter` is there to get started easily. It is a pre-configured `KMBloomFilter` that uses two hash functions which proved sufficient for our tests.

```rust
use bloom_filter_simple::{BloomFilter,DefaultBloomFilter};

fn main() {
    // We plan on storing at most 10 elements
    let desired_capacity = 10;

    // The chance of a false positive increases with each inserted element. This parameter
    // specifies that it should be less than 0.01% (0.0001) when the desired capacity has
    // been reached.
    // In other words, the chance that the bloom filter returns true when checking whether a
    // novel element has been inserted before is less than 0.01% (0.0001).
    let desired_fp_probability = 0.0001;

    let mut filter = DefaultBloomFilter::new(desired_capacity, desired_fp_probability);

    // You can insert any type implementing the Hash trait. The bloom filter does not store the
    // inserted elements but only their hashes. Hence, there is no transfer of ownership required.
    filter.insert(&5i32);
    filter.insert(&"Some text");
    filter.insert(&10_000usize);

    // You can check whether a value has been inserted into the filter before.
    assert_eq!(false, filter.contains(&3));
    assert_eq!(true, filter.contains(&5));
    assert_eq!(true, filter.contains(&"Some text"));
}
```

## `KMBloomFilter`

The `KMBloomFilter` lets you choose which hash functions should be used.

```rust
KMBloomFilter<AHasher, DefaultHasher> = KMBloomFilter::new(desired_capacity, desired_fp_probability);
```

## `SeededBloomFilter`

The `SeededBloomFilter` requires no configuration as it uses only one specific hash function which is seeded automatically.

```rust
SeededBloomFilter::new(desired_capacity, desired_fp_probability);
```