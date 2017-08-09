pub fn set_bit(mut val: u8, bit_nr: usize) -> u8 {
    val |= bit_mask(bit_nr) as u8;
    val
}

pub fn test_bit(val: u8, bit_nr: usize) -> bool {
    (val & bit_mask(bit_nr) as u8) != 0
}

fn bit_mask(bit: usize) -> usize {
    (1 << bit)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_bit() {
        assert_eq!(super::test_bit(0b10, 0), false);
        assert_eq!(super::test_bit(0b10, 1), true);
    }

    #[test]
    fn set_bit() {
        assert_eq!(super::set_bit(0x0, 0), 0b01);
        assert_eq!(super::set_bit(0x0, 1), 0b10);
    }
}
