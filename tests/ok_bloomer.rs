use std::{collections::hash_map::DefaultHasher, hash::Hasher};

use bloom_filter::seeded_bloomfilter::SeededBloomFilter;
use bloom_filter::{BloomFilter, DefaultBloomFilter};
use bloomfilter::Bloom;
use rand::{distributions::Uniform, prelude::StdRng, Rng, SeedableRng};
use xxhash_rust::xxh3;

#[test]
fn bloomer() {
    let mut bloomer = DefaultBloomFilter::new(3, 0.7);

    println!("Bloomer before insert: {:?}", bloomer);
    println!("Probability: {}", bloomer.false_positive_probability());

    bloomer.insert(&5);

    println!("Bloomer after five: {:?}", bloomer);
    println!("Probability: {}", bloomer.false_positive_probability());

    bloomer.insert(&3);

    println!("Bloomer after three: {:?}", bloomer);
    println!("Probability: {}", bloomer.false_positive_probability());

    assert!(bloomer.check(&3));
    assert!(bloomer.check(&5));
}

#[test]
#[ignore]
fn false_positive_probability_extern() {
    let desired_capacity = 10_000_000;
    let false_positive_probability = 0.001;
    let relative_error_margin = 0.001;
    let bloomer: Bloom<usize> =
        Bloom::new_for_fp_rate(desired_capacity, false_positive_probability);

    test_extern_bloomfilter(
        desired_capacity,
        false_positive_probability,
        bloomer,
        relative_error_margin,
    );
}

#[test]
fn false_positive_probability_seeded() {
    let desired_capacity = 1_000_000;
    let false_positive_probability = 0.001;
    let relative_error_margin = 0.06;
    let bloomer = SeededBloomFilter::new(desired_capacity, false_positive_probability);

    test_seeded_bloom_filter_probability(
        desired_capacity,
        false_positive_probability,
        bloomer,
        relative_error_margin,
    );
}
#[test]
fn false_positive_probability_test_default_fnv() {
    let desired_capacity = 1_000_000;
    let false_positive_probability = 0.001;
    let relative_error_margin = 0.034;
    let bloomer: BloomFilter<DefaultHasher, fnv::FnvHasher> =
        BloomFilter::new(desired_capacity, false_positive_probability);

    test_bloom_filter_probability(
        desired_capacity,
        false_positive_probability,
        bloomer,
        relative_error_margin,
    );
}

#[test]
fn false_positive_probability_default_ahash() {
    let desired_capacity = 1_000_000;
    let false_positive_probability = 0.001;
    let relative_error_margin = 0.028;
    let bloomer: BloomFilter<DefaultHasher, ahash::AHasher> =
        BloomFilter::new(desired_capacity, false_positive_probability);

    test_bloom_filter_probability(
        desired_capacity,
        false_positive_probability,
        bloomer,
        relative_error_margin,
    );
}

#[test]
fn false_positive_probability_xx_default() {
    let desired_capacity = 1_000_000;
    let false_positive_probability = 0.001;
    let relative_error_margin = 0.005;
    let bloomer: BloomFilter<xxh3::Xxh3, DefaultHasher> =
        BloomFilter::new(desired_capacity, false_positive_probability);

    test_bloom_filter_probability(
        desired_capacity,
        false_positive_probability,
        bloomer,
        relative_error_margin,
    );
}

#[test]
fn false_positive_probability_test_random_default_fnv() {
    let desired_capacity = 1_000_000;
    let false_positive_probability = 0.001;
    let relative_error_margin = 0.04;
    let bloomer: BloomFilter<DefaultHasher, fnv::FnvHasher> =
        BloomFilter::new(desired_capacity, false_positive_probability);

    test_bloom_filter_probability_random(
        desired_capacity,
        false_positive_probability,
        bloomer,
        relative_error_margin,
    );
}

#[test]
fn false_positive_probability_random_default_ahash() {
    let desired_capacity = 1_000_000;
    let false_positive_probability = 0.001;
    let relative_error_margin = 0.034;
    let bloomer: BloomFilter<DefaultHasher, ahash::AHasher> =
        BloomFilter::new(desired_capacity, false_positive_probability);

    test_bloom_filter_probability_random(
        desired_capacity,
        false_positive_probability,
        bloomer,
        relative_error_margin,
    );
}

#[test]
fn false_positive_probability_random_ahash_default() {
    let desired_capacity = 1_000_000;
    let false_positive_probability = 0.001;
    let relative_error_margin = 0.00002;
    let bloomer: BloomFilter<ahash::AHasher, DefaultHasher> =
        BloomFilter::new(desired_capacity, false_positive_probability);

    test_bloom_filter_probability_random(
        desired_capacity,
        false_positive_probability,
        bloomer,
        relative_error_margin,
    );
}

fn test_bloom_filter_probability<H1, H2>(
    desired_capacity: usize,
    false_positive_probability: f64,
    mut bloomer: BloomFilter<H1, H2>,
    relative_error_margin: f64,
) where
    H1: Hasher + Default,
    H2: Hasher + Default,
{
    let allowed_probability = false_positive_probability * (1.0 + relative_error_margin);
    for i in 1..desired_capacity {
        bloomer.insert(&i);
        assert!(bloomer.false_positive_probability() < allowed_probability);
    }

    let true_checks = (0..(desired_capacity * 2))
        .map(|i| bloomer.check(&i))
        .filter(|c| *c)
        .count();

    println!("Desired capacity: {}", desired_capacity);
    println!(
        "Desired false positive probability: {}",
        false_positive_probability
    );
    println!("Calculated hash count: {}", bloomer.hash_count());
    println!("Positive check count: {}", true_checks);
    println!(
        "Calculated false positive probability: {} ({})",
        bloomer.false_positive_probability(),
        allowed_probability,
    );
    println!(
        "Tested false positive probability: {} ({})",
        (true_checks as f64 - desired_capacity as f64) / desired_capacity as f64,
        allowed_probability
    );
    assert!(true_checks < (desired_capacity as f64 * (1.0 + allowed_probability)) as usize);
}

fn test_bloom_filter_probability_random<H1, H2>(
    desired_capacity: usize,
    false_positive_probability: f64,
    mut bloomer: BloomFilter<H1, H2>,
    relative_error_margin: f64,
) where
    H1: Hasher + Default,
    H2: Hasher + Default,
{
    let seed = [0xb7u8; 32];
    let mut rng = StdRng::from_seed(seed);
    let distribution = Uniform::new(u64::MIN, u64::MAX);
    let allowed_probability = false_positive_probability * (1.0 + relative_error_margin);
    for _ in 1..desired_capacity {
        bloomer.insert(&rng.sample(distribution));
        assert!(bloomer.false_positive_probability() < allowed_probability);
    }

    let mut rng = rand::rngs::StdRng::from_seed(seed);
    let true_checks = (0..(desired_capacity * 2))
        .map(|_| bloomer.check(&rng.sample(distribution)))
        .filter(|c| *c)
        .count();

    println!("Desired capacity: {}", desired_capacity);
    println!(
        "Desired false positive probability: {}",
        false_positive_probability
    );
    println!("Calculated hash count: {}", bloomer.hash_count());
    println!("Positive check count: {}", true_checks);
    println!(
        "Calculated false positive probability: {} ({})",
        bloomer.false_positive_probability(),
        allowed_probability,
    );
    println!(
        "Tested false positive probability: {} ({})",
        (true_checks as f64 - desired_capacity as f64) / desired_capacity as f64,
        allowed_probability
    );
    assert!(true_checks < (desired_capacity as f64 * (1.0 + allowed_probability)) as usize);
}

fn test_extern_bloomfilter(
    desired_capacity: usize,
    false_positive_probability: f64,
    mut bloomer: Bloom<usize>,
    relative_error_margin: f64,
) {
    let allowed_probability = false_positive_probability * (1.0 + relative_error_margin);
    for i in 0..desired_capacity {
        bloomer.set(&i);
    }

    let true_checks = (0..(desired_capacity * 2))
        .map(|i| bloomer.check(&i))
        .filter(|c| *c)
        .count();

    println!("Desired capacity: {}", desired_capacity);
    println!(
        "Desired false positive probability: {}",
        false_positive_probability
    );
    println!("Positive check count: {}", true_checks);
    println!(
        "Tested false positive probability: {} ({})",
        (true_checks as f64 - desired_capacity as f64) / desired_capacity as f64,
        allowed_probability
    );
    assert!(true_checks < (desired_capacity as f64 * (1.0 + allowed_probability)) as usize);
}

fn test_seeded_bloom_filter_probability(
    desired_capacity: usize,
    false_positive_probability: f64,
    mut bloomer: SeededBloomFilter,
    relative_error_margin: f64,
) {
    let allowed_probability = false_positive_probability * (1.0 + relative_error_margin);
    for i in 0..desired_capacity {
        bloomer.insert(&i);
        assert!(bloomer.false_positive_probability() < allowed_probability);
    }

    let true_checks = (0..(desired_capacity * 2))
        .map(|i| bloomer.check(&i))
        .filter(|c| *c)
        .count();

    println!("Desired capacity: {}", desired_capacity);
    println!(
        "Desired false positive probability: {}",
        false_positive_probability
    );
    println!("Calculated hash count: {}", bloomer.hash_count());
    println!("Positive check count: {}", true_checks);
    println!(
        "Calculated false positive probability: {} ({})",
        bloomer.false_positive_probability(),
        allowed_probability,
    );
    println!(
        "Tested false positive probability: {} ({})",
        (true_checks as f64 - desired_capacity as f64) / desired_capacity as f64,
        allowed_probability
    );
    assert!(true_checks < (desired_capacity as f64 * (1.0 + allowed_probability)) as usize);
}
