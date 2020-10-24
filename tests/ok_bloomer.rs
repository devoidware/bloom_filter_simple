use std::{collections::hash_map::DefaultHasher, hash::Hasher};

use bloom_filter::BloomFilter;

#[test]
fn bloomer() {
    let mut bloomer: BloomFilter<DefaultHasher, fnv::FnvHasher> = BloomFilter::new(3, 0.7);

    println!("Bloomer before insert: {:?}", bloomer);
    println!("Probability: {}", bloomer.false_positive_probability());

    bloomer.insert(5);

    println!("Bloomer after five: {:?}", bloomer);
    println!("Probability: {}", bloomer.false_positive_probability());

    bloomer.insert(3);

    println!("Bloomer after three: {:?}", bloomer);
    println!("Probability: {}", bloomer.false_positive_probability());

    assert!(bloomer.check(&3));
    assert!(bloomer.check(&5));
}

#[test]
fn false_positive_probability_default() {
    let desired_capacity = 1_000_000;
    let false_positive_probability = 0.001;
    let relative_error_margin = 0.001;
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
fn false_positive_probability_test_fnv() {
    let desired_capacity = 1_000_000;
    let false_positive_probability = 0.001;
    let relative_error_margin = 0.30;
    let bloomer: BloomFilter<DefaultHasher, fnv::FnvHasher> =
        BloomFilter::new(desired_capacity, false_positive_probability);

    test_bloom_filter_probability(
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
    for i in 0..desired_capacity {
        bloomer.insert(i);
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
