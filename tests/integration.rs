use ahash::AHasher;
#[cfg(feature = "union")]
use bloom_filter_simple::Union;
use bloom_filter_simple::{
    BloomFilter, DefaultBloomFilter, HasherBuilder, KMBloomFilter, SeededKMBloomFilter,
    SingleHasherBloomFilter,
};
use highway::{HighwayHasher, Key};
use rand::{distributions::Uniform, prelude::StdRng, Rng, SeedableRng};
use std::{collections::hash_map::DefaultHasher, hash::Hasher};
use xxhash_rust::xxh3;

#[test]
fn bloom_filter() {
    let mut bloom_filter = DefaultBloomFilter::new(3, 0.7);

    println!("Bloom_filter before insert: {:?}", bloom_filter);
    println!(
        "Probability: {}",
        bloom_filter.approximate_current_false_positive_probability()
    );

    bloom_filter.insert(&5);

    println!("Bloom_filter after five: {:?}", bloom_filter);
    println!(
        "Probability: {}",
        bloom_filter.approximate_current_false_positive_probability()
    );

    bloom_filter.insert(&3);

    println!("Bloom_filter after three: {:?}", bloom_filter);
    println!(
        "Probability: {}",
        bloom_filter.approximate_current_false_positive_probability()
    );

    assert!(bloom_filter.contains(&3));
    assert!(bloom_filter.contains(&5));
}

#[test]
fn false_positive_probability_seeded() {
    let desired_capacity = 1_000_000;
    let false_positive_probability = 0.001;
    let relative_error_margin = 0.06;
    let bloom_filter = SingleHasherBloomFilter::new(desired_capacity, false_positive_probability);

    test_single_hasher_bloom_filter_probability(
        desired_capacity,
        false_positive_probability,
        bloom_filter,
        relative_error_margin,
    );
}

#[test]
fn false_positive_probability_default_ahash() {
    let desired_capacity = 1_000_000;
    let false_positive_probability = 0.001;
    let relative_error_margin = 0.028;
    let bloom_filter: KMBloomFilter<DefaultHasher, ahash::AHasher> =
        KMBloomFilter::new(desired_capacity, false_positive_probability);

    test_bloom_filter_probability(
        desired_capacity,
        false_positive_probability,
        bloom_filter,
        relative_error_margin,
    );
}

#[test]
fn false_positive_probability_xx_default() {
    let desired_capacity = 1_000_000;
    let false_positive_probability = 0.001;
    let relative_error_margin = 0.005;
    let bloom_filter: KMBloomFilter<xxh3::Xxh3, DefaultHasher> =
        KMBloomFilter::new(desired_capacity, false_positive_probability);

    test_bloom_filter_probability(
        desired_capacity,
        false_positive_probability,
        bloom_filter,
        relative_error_margin,
    );
}

#[test]
fn false_positive_probability_random_default_ahash() {
    let desired_capacity = 1_000_000;
    let false_positive_probability = 0.001;
    let relative_error_margin = 0.034;
    let bloom_filter: KMBloomFilter<DefaultHasher, ahash::AHasher> =
        KMBloomFilter::new(desired_capacity, false_positive_probability);

    test_bloom_filter_probability_random(
        desired_capacity,
        false_positive_probability,
        bloom_filter,
        relative_error_margin,
    );
}

#[test]
fn false_positive_probability_random_ahash_default() {
    let desired_capacity = 1_000_000;
    let false_positive_probability = 0.001;
    let relative_error_margin = 0.01;
    let bloom_filter: KMBloomFilter<ahash::AHasher, DefaultHasher> =
        KMBloomFilter::new(desired_capacity, false_positive_probability);

    test_bloom_filter_probability_random(
        desired_capacity,
        false_positive_probability,
        bloom_filter,
        relative_error_margin,
    );
}

#[test]
fn false_positive_probability_random_seeded_ahash() {
    struct AHasherBuilder;

    impl HasherBuilder<AHasher, (u128, u128)> for AHasherBuilder {
        fn new_with_seed(seed: (u128, u128)) -> AHasher {
            AHasher::new_with_keys(seed.0, seed.1)
        }
    }
    let desired_capacity = 1_000_000;
    let false_positive_probability = 0.001;
    let relative_error_margin = 0.01;
    let bloom_filter = SeededKMBloomFilter::new_with_seeds::<AHasherBuilder, AHasherBuilder>(
        desired_capacity,
        false_positive_probability,
        (1, 1),
        (2, 2),
    );

    test_bloom_filter_probability_random(
        desired_capacity,
        false_positive_probability,
        bloom_filter,
        relative_error_margin,
    );
}

#[test]
fn false_positive_probability_random_highway() {
    struct HighwayHasherBuilder;
    #[derive(Clone, Debug)]
    struct KeyComparator(Key);
    impl PartialEq for KeyComparator {
        fn eq(&self, other: &Self) -> bool {
            self.0 .0 == other.0 .0
        }
    }

    impl HasherBuilder<HighwayHasher, KeyComparator> for HighwayHasherBuilder {
        fn new_with_seed(seed: KeyComparator) -> HighwayHasher {
            HighwayHasher::new(seed.0)
        }
    }
    let desired_capacity = 1_000_000;
    let false_positive_probability = 0.001;
    let relative_error_margin = 0.01;
    let bloom_filter =
        SeededKMBloomFilter::new_with_seeds::<HighwayHasherBuilder, HighwayHasherBuilder>(
            desired_capacity,
            false_positive_probability,
            KeyComparator(Key([1, 2, 3, 4])),
            KeyComparator(Key([5, 6, 7, 8])),
        );

    test_bloom_filter_probability_random(
        desired_capacity,
        false_positive_probability,
        bloom_filter,
        relative_error_margin,
    );
}

fn test_bloom_filter_probability<H1, H2>(
    desired_capacity: usize,
    false_positive_probability: f64,
    mut bloom_filter: KMBloomFilter<H1, H2>,
    relative_error_margin: f64,
) where
    H1: Hasher + Default + Clone,
    H2: Hasher + Default + Clone,
{
    let allowed_probability = false_positive_probability * (1.0 + relative_error_margin);
    for i in 0..desired_capacity {
        bloom_filter.insert(&i);
    }
    assert!(bloom_filter.approximate_current_false_positive_probability() <= allowed_probability);

    let true_checks = (desired_capacity..(desired_capacity * 2))
        .map(|i| bloom_filter.contains(&i))
        .filter(|c| *c)
        .count();

    println!("Desired capacity: {}", desired_capacity);
    println!(
        "Desired false positive probability: {}",
        false_positive_probability
    );
    println!(
        "Calculated element count: {}",
        bloom_filter.approximate_element_count()
    );
    println!("Positive check count: {}", true_checks);
    println!(
        "Calculated false positive probability: {} ({})",
        bloom_filter.approximate_current_false_positive_probability(),
        allowed_probability,
    );
    println!(
        "Tested false positive probability: {} ({})",
        true_checks as f64 / desired_capacity as f64,
        allowed_probability
    );
    assert!(true_checks <= (desired_capacity as f64 * (1.0 + allowed_probability)) as usize);
}

fn test_bloom_filter_probability_random<H1, H2, S1, S2>(
    desired_capacity: usize,
    false_positive_probability: f64,
    mut bloom_filter: SeededKMBloomFilter<H1, H2, S1, S2>,
    relative_error_margin: f64,
) where
    H1: Hasher + Clone,
    H2: Hasher + Clone,
    S1: Clone + PartialEq,
    S2: Clone + PartialEq,
{
    let seed = [0xb7u8; 32];
    let mut rng = StdRng::from_seed(seed);
    let distribution = Uniform::new(u64::MIN, u64::MAX);
    let allowed_probability = false_positive_probability * (1.0 + relative_error_margin);
    for _ in 0..desired_capacity {
        bloom_filter.insert(&rng.sample(distribution));
    }
    assert!(bloom_filter.approximate_current_false_positive_probability() <= allowed_probability);

    let seed = [0x3Fu8; 32];
    let mut rng = rand::rngs::StdRng::from_seed(seed);
    let true_checks = (0..desired_capacity)
        .map(|_| bloom_filter.contains(&rng.sample(distribution)))
        .filter(|c| *c)
        .count();

    println!("Desired capacity: {}", desired_capacity);
    println!(
        "Desired false positive probability: {}",
        false_positive_probability
    );
    println!(
        "Calculated element count: {}",
        bloom_filter.approximate_element_count()
    );
    println!("Positive check count: {}", true_checks);
    println!(
        "Calculated false positive probability: {} ({})",
        bloom_filter.approximate_current_false_positive_probability(),
        allowed_probability,
    );

    println!(
        "Tested false positive probability: {} ({})",
        true_checks as f64 / desired_capacity as f64,
        allowed_probability
    );
    assert!(true_checks <= (desired_capacity as f64 * (1.0 + allowed_probability)) as usize);
}

fn test_single_hasher_bloom_filter_probability(
    desired_capacity: usize,
    false_positive_probability: f64,
    mut bloom_filter: SingleHasherBloomFilter,
    relative_error_margin: f64,
) {
    let allowed_probability = false_positive_probability * (1.0 + relative_error_margin);
    for i in 0..desired_capacity {
        bloom_filter.insert(&i);
    }
    assert!(bloom_filter.approximate_current_false_positive_probability() <= allowed_probability);

    let true_checks = (desired_capacity..(desired_capacity * 2))
        .map(|i| bloom_filter.contains(&i))
        .filter(|c| *c)
        .count();

    println!("Desired capacity: {}", desired_capacity);
    println!(
        "Desired false positive probability: {}",
        false_positive_probability
    );
    println!("Positive check count: {}", true_checks);
    println!(
        "Calculated false positive probability: {} ({})",
        bloom_filter.approximate_current_false_positive_probability(),
        allowed_probability,
    );
    println!(
        "Tested false positive probability: {} ({})",
        true_checks as f64 / desired_capacity as f64,
        allowed_probability
    );
    assert!(true_checks <= (desired_capacity as f64 * (1.0 + allowed_probability)) as usize);
}

#[test]
fn test_bloom_filter_with_strings() {
    let mut bloom_filter = DefaultBloomFilter::new(1000, 0.001);

    bloom_filter.insert(&"This");
    bloom_filter.insert(&"is");
    bloom_filter.insert(&"a");
    bloom_filter.insert(&"simple");
    bloom_filter.insert(&"test");
    bloom_filter.insert(&"!");

    assert_eq!(false, bloom_filter.contains(&"Not"));
    assert_eq!(true, bloom_filter.contains(&"a"));
    assert_eq!(false, bloom_filter.contains(&"single"));
    assert_eq!(false, bloom_filter.contains(&"problem"));
    assert_eq!(false, bloom_filter.contains(&"found"));
    assert_eq!(true, bloom_filter.contains(&"!"));
}

#[test]
#[ignore]
fn insert_and_check_its_there_with_millions_of_values() {
    let n_values = 10_000_000;
    let mut bloom_filter = DefaultBloomFilter::new(n_values, 0.001);

    for i in 0..n_values {
        bloom_filter.insert(&i);
    }

    for i in 0..n_values {
        assert!(bloom_filter.contains(&i));
    }
}

#[cfg(feature = "union")]
#[test]
fn km_bloom_filter_union_test() {
    let desired_capacity = 1_000_000;
    let false_positive_probability = 0.001;
    let relative_error_margin = 0.00002;
    let mut bloom_filter_a: KMBloomFilter<ahash::AHasher, DefaultHasher> =
        KMBloomFilter::new(desired_capacity, false_positive_probability);
    let mut bloom_filter_b: KMBloomFilter<ahash::AHasher, DefaultHasher> =
        KMBloomFilter::new(desired_capacity, false_positive_probability);

    let seed = [0xb7u8; 32];
    let mut rng = StdRng::from_seed(seed);
    let distribution = Uniform::new(u64::MIN, u64::MAX);

    let allowed_probability = false_positive_probability * (1.0 + relative_error_margin);

    for _ in 0..(desired_capacity / 2) {
        bloom_filter_a.insert(&rng.sample(distribution));
    }
    assert!(bloom_filter_a.approximate_current_false_positive_probability() <= allowed_probability);

    for _ in 0..(desired_capacity / 2) {
        bloom_filter_b.insert(&rng.sample(distribution));
    }
    assert!(bloom_filter_b.approximate_current_false_positive_probability() <= allowed_probability);

    let bloom_filter = bloom_filter_a.union(&bloom_filter_b);

    let seed = [0x91u8; 32];
    let mut rng = StdRng::from_seed(seed);
    let true_checks = (0..desired_capacity)
        .map(|_| bloom_filter.contains(&rng.sample(distribution)))
        .filter(|c| *c)
        .count();

    println!("Desired capacity: {}", desired_capacity);
    println!(
        "Desired false positive probability: {}",
        false_positive_probability
    );
    println!(
        "Calculated element count: {}",
        bloom_filter.approximate_element_count()
    );
    println!("Positive check count: {}", true_checks);
    println!(
        "Calculated false positive probability: {} ({})",
        bloom_filter.approximate_current_false_positive_probability(),
        allowed_probability,
    );
    println!(
        "Tested false positive probability: {} ({})",
        true_checks as f64 / desired_capacity as f64,
        allowed_probability
    );
    assert!(true_checks <= (desired_capacity as f64 * (1.0 + allowed_probability)) as usize);
}

#[test]
fn km_bloom_filter_intersect_test() {
    let desired_capacity = 1_000_000;
    let false_positive_probability = 0.0001;
    let relative_error_margin = 0.07;
    let mut bloom_filter_a: KMBloomFilter<ahash::AHasher, DefaultHasher> = KMBloomFilter::new(
        (desired_capacity as f64 * 1.5) as usize,
        false_positive_probability,
    );
    let mut bloom_filter_b: KMBloomFilter<ahash::AHasher, DefaultHasher> = KMBloomFilter::new(
        (desired_capacity as f64 * 1.5) as usize,
        false_positive_probability,
    );

    let seed = [0x2Au8; 32];
    let mut rng = StdRng::from_seed(seed);
    let distribution = Uniform::new(u64::MIN, u64::MAX);

    let allowed_probability = false_positive_probability * (1.0 + relative_error_margin);

    for _ in 0..desired_capacity {
        let value = rng.sample(distribution);
        bloom_filter_a.insert(&value);
        bloom_filter_b.insert(&value);
    }

    for _ in 0..(desired_capacity / 2) {
        bloom_filter_a.insert(&rng.sample(distribution));
    }

    for _ in 0..(desired_capacity / 2) {
        bloom_filter_b.insert(&rng.sample(distribution));
    }

    let bloom_filter = bloom_filter_a.intersect(&bloom_filter_b);

    let seed = [0xCAu8; 32];
    let mut rng = StdRng::from_seed(seed);
    let true_checks = (0..desired_capacity)
        .map(|_| bloom_filter.contains(&rng.sample(distribution)))
        .filter(|c| *c)
        .count();

    println!("Desired capacity: {}", desired_capacity);
    println!(
        "Desired false positive probability: {}",
        false_positive_probability
    );
    println!(
        "Calculated element count: {}",
        bloom_filter.approximate_element_count()
    );
    println!("Positive check count: {}", true_checks);
    println!(
        "Calculated false positive probability: {} ({})",
        bloom_filter.approximate_current_false_positive_probability(),
        allowed_probability,
    );
    println!(
        "Tested false positive probability: {} ({})",
        true_checks as f64 / desired_capacity as f64,
        allowed_probability
    );
    assert!(true_checks <= (desired_capacity as f64 * (1.0 + allowed_probability)) as usize);
}
