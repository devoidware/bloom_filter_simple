use std::collections::hash_map::DefaultHasher;

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
