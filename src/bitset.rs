pub struct Bitset {
    bytes: Vec<u8>,
    length: usize,
}

impl Bitset {
    pub fn new(length: usize) -> Self {
        todo!()
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn set(&mut self, index: usize, value: bool) {
        todo!()
    }

    pub fn get(&self, index: usize) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bitset_with_length() {
        let bitset = Bitset::new(1);
        assert_eq!(1, bitset.length);
        assert_eq!(1, bitset.len());
        assert_eq!(1, bitset.bytes.len());

        let bitset = Bitset::new(8);
        assert_eq!(8, bitset.length);
        assert_eq!(8, bitset.len());
        assert_eq!(1, bitset.bytes.len());

        let bitset = Bitset::new(9);
        assert_eq!(9, bitset.length);
        assert_eq!(9, bitset.len());
        assert_eq!(2, bitset.bytes.len());
    }

    #[test]
    #[should_panic(expected = "at least one element")]
    fn bitset_must_not_be_empty() {
        Bitset::new(0);
    }

    #[test]
    fn set_first_bit_only() {
        let mut bitset = Bitset::new(3);
        bitset.set(0, true);
        assert_eq!(true, bitset.get(0));
        assert_eq!(false, bitset.get(1));
        assert_eq!(false, bitset.get(2));
    }

    #[test]
    fn set_last_bit_only() {
        let mut bitset = Bitset::new(9);
        bitset.set(8, true);
        for i in 0..8 {
            assert_eq!(false, bitset.get(i));
        }
        assert_eq!(true, bitset.get(8));
    }

    #[test]
    #[should_panic(expected = "out of range")]
    fn must_access_correct_index() {
        let mut bitset = Bitset::new(5);
        bitset.set(5, true);
    }
}
