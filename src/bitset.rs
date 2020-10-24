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

    fn get_number_of_set_bits(bitset: &Bitset) -> usize {
        let mut i = 0;
        for j in 0..bitset.len() {
            if bitset.get(j) {
                i += 1;
            }
        }
        i
    }
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
    fn must_set_with_correct_index() {
        Bitset::new(5).set(5, true);
    }

    #[test]
    #[should_panic(expected = "out of range")]
    fn must_get_with_correct_index() {
        Bitset::new(12).get(12);
    }

    #[test]
    fn set_and_unset_possible() {
        let mut bitset = Bitset::new(24);
        for i in 0..24 {
            assert_eq!(false, bitset.get(i));
        }

        bitset.set(0, true);
        bitset.set(7, true);
        bitset.set(8, true);
        bitset.set(23, true);

        assert_eq!(true, bitset.get(0));
        assert_eq!(true, bitset.get(7));
        assert_eq!(true, bitset.get(8));
        assert_eq!(true, bitset.get(23));

        bitset.set(0, true);
        bitset.set(8, false);
        bitset.set(23, false);

        assert_eq!(true, bitset.get(0));
        assert_eq!(true, bitset.get(7));
        assert_eq!(false, bitset.get(8));
        assert_eq!(false, bitset.get(23));
    }

    #[test]
    fn set_each_bit_one_by_one() {
        let mut bitset = Bitset::new(9);
        assert_eq!(0, get_number_of_set_bits(&bitset));

        bitset.set(0, true);
        assert_eq!(true, bitset.get(0));
        assert_eq!(1, get_number_of_set_bits(&bitset));

        bitset.set(1, true);
        assert_eq!(true, bitset.get(1));
        assert_eq!(2, get_number_of_set_bits(&bitset));

        bitset.set(2, true);
        assert_eq!(true, bitset.get(2));
        assert_eq!(3, get_number_of_set_bits(&bitset));

        bitset.set(3, true);
        assert_eq!(true, bitset.get(3));
        assert_eq!(4, get_number_of_set_bits(&bitset));

        bitset.set(4, true);
        assert_eq!(true, bitset.get(4));
        assert_eq!(5, get_number_of_set_bits(&bitset));

        bitset.set(5, true);
        assert_eq!(true, bitset.get(5));
        assert_eq!(6, get_number_of_set_bits(&bitset));

        bitset.set(6, true);
        assert_eq!(true, bitset.get(6));
        assert_eq!(7, get_number_of_set_bits(&bitset));

        bitset.set(7, true);
        assert_eq!(true, bitset.get(7));
        assert_eq!(8, get_number_of_set_bits(&bitset));

        bitset.set(8, true);
        assert_eq!(true, bitset.get(8));
        assert_eq!(9, get_number_of_set_bits(&bitset));
    }

    #[test]
    fn unset_each_bit_one_by_one() {
        let mut bitset = Bitset::new(9);
        for i in 0..bitset.len() {
            bitset.set(i, true);
        }
        assert_eq!(9, get_number_of_set_bits(&bitset));

        bitset.set(0, false);
        assert_eq!(false, bitset.get(0));
        assert_eq!(8, get_number_of_set_bits(&bitset));

        bitset.set(1, false);
        assert_eq!(false, bitset.get(1));
        assert_eq!(7, get_number_of_set_bits(&bitset));

        bitset.set(2, false);
        assert_eq!(false, bitset.get(2));
        assert_eq!(6, get_number_of_set_bits(&bitset));

        bitset.set(3, false);
        assert_eq!(false, bitset.get(3));
        assert_eq!(5, get_number_of_set_bits(&bitset));

        bitset.set(4, false);
        assert_eq!(false, bitset.get(4));
        assert_eq!(4, get_number_of_set_bits(&bitset));

        bitset.set(5, false);
        assert_eq!(false, bitset.get(5));
        assert_eq!(3, get_number_of_set_bits(&bitset));

        bitset.set(6, false);
        assert_eq!(false, bitset.get(6));
        assert_eq!(2, get_number_of_set_bits(&bitset));

        bitset.set(7, false);
        assert_eq!(false, bitset.get(7));
        assert_eq!(1, get_number_of_set_bits(&bitset));

        bitset.set(8, false);
        assert_eq!(false, bitset.get(8));
        assert_eq!(0, get_number_of_set_bits(&bitset));
    }
}
