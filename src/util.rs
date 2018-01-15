use byteorder::{ByteOrder, LittleEndian};

pub fn set_bit(mut val: u8, bit_nr: usize) -> u8 {
    val |= bit_mask(bit_nr) as u8;
    val
}

pub fn set_bit_16(mut val: u16, bit_nr: usize) -> u16 {
    val |= bit_mask(bit_nr) as u16;
    val
}

pub fn test_bit(val: u8, bit_nr: usize) -> bool {
    test_bit_16(u16::from(val), bit_nr)
}

pub fn test_bit_16(val: u16, bit_nr: usize) -> bool {
    (val & bit_mask(bit_nr) as u16) != 0
}

fn bit_mask(bit: usize) -> usize {
    (1 << bit)
}

pub fn u16_to_u8(words: &[u16]) -> Vec<u8> {
    let mut bytes = vec![0; 2 * words.len()];
    LittleEndian::write_u16_into(words, &mut bytes);
    bytes
}

pub fn u8_to_u16(bytes: &[u8]) -> Vec<u16> {
    let mut src = vec![];
    src.extend_from_slice(bytes);
    let mut cnt = src.len();
    if (cnt % 2) != 0 {
        cnt += 1;
        src.push(0);
    }
    let cnt = cnt / 2;
    let mut words = vec![0; cnt];
    LittleEndian::read_u16_into(&src, &mut words);
    words
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

    #[test]
    fn u16_to_u8() {
        assert_eq!(super::u16_to_u8(&[]), vec![]);
        assert_eq!(super::u16_to_u8(&[0xABCD]), vec![0xCD, 0xAB]);
        assert_eq!(
            super::u16_to_u8(&[0xAB, 0xCD]),
            vec![0xAB, 0x00, 0xCD, 0x00]
        );
    }

    #[test]
    fn u8_to_u16() {
        assert_eq!(super::u8_to_u16(&[]), vec![]);
        assert_eq!(super::u8_to_u16(&[0xAB]), vec![0xAB]);
        assert_eq!(super::u8_to_u16(&[0xA, 0xB]), vec![0x0B0A]);
        assert_eq!(super::u8_to_u16(&[0xA, 0xB, 0xC]), vec![0x0B0A, 0xC]);
    }
}
