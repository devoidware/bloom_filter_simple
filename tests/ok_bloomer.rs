use std::{collections::hash_map::DefaultHasher, hash::Hasher};

use bloom_filter::BloomFilter;

#[test]
fn bloomer() {
    let mut bloomer: BloomFilter<DefaultHasher> = BloomFilter::new(3, 0.7);

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
    let false_positive_probability = 0.25;
    let bloomer: BloomFilter<DefaultHasher> =
        BloomFilter::new(desired_capacity, false_positive_probability);

    test_bloom_filter_probability(desired_capacity, false_positive_probability, bloomer);
}

#[test]
fn false_positive_probability_test_fnv() {
    let desired_capacity = 1_000_000;
    let false_positive_probability = 0.25;
    let bloomer: BloomFilter<fnv::FnvHasher> =
        BloomFilter::new(desired_capacity, false_positive_probability);

    test_bloom_filter_probability(desired_capacity, false_positive_probability, bloomer);
}

fn test_bloom_filter_probability<H>(
    desired_capacity: usize,
    false_positive_probability: f64,
    mut bloomer: BloomFilter<H>,
) where
    H: Hasher + Default,
{
    for i in 0..desired_capacity {
        bloomer.insert(i);
        assert!(bloomer.false_positive_probability() < false_positive_probability);
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
        "Calculated false positive probability: {}",
        bloomer.false_positive_probability()
    );
    println!(
        "Tested false positive probability: {}",
        (true_checks as f64 - desired_capacity as f64) / desired_capacity as f64
    );
    assert!(true_checks < (desired_capacity as f64 * (1.0 + false_positive_probability)) as usize);
}
