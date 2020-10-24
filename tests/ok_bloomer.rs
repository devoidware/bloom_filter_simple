use bloom_filter::BloomFilter;
use bloom_filter::ResettableHasher;
use std::collections::hash_map::DefaultHasher;

#[test]
fn bloomer() {
    let hashers: Vec<Box<dyn ResettableHasher>> = vec![Box::new(DefaultHasher::new())];
    let mut bloomer = BloomFilter::new(3, hashers);
    println!("Probability: {}", bloomer.false_positive_probability());
    println!("Bloomer before insert: {:?}", bloomer);
    bloomer.insert(5);
    println!("Bloomer after five: {:?}", bloomer);
    bloomer.insert(3);
    println!("Bloomer after three: {:?}", bloomer);
    println!("Probability: {}", bloomer.false_positive_probability());
    assert!(bloomer.check(&3));
    assert!(bloomer.check(&5));
}
