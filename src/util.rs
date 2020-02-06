use super::*;
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

pub fn shift_data(data: &[u16]) -> Vec<u16> {
    let buf = u16_to_u8(data);
    let buf = &buf[1..]; // drop first byte
    let mut shifted = vec![];
    shifted.extend_from_slice(buf);
    shifted.push(0);
    u8_to_u16(&shifted)
}

pub fn analog_ui_value_to_u16(v: f32, range: &AnalogUIRange, format: &DataFormat) -> u16 {
    let factor = format.factor();
    use crate::AnalogUIRange::*;

    #[rustfmt::skip]
    let v = match *range {
        mA0To20       => (factor * v / 20.0),
        mA4To20       => (factor * (v - 4.0) / 16.0),
        V0To10        |
        VMinus10To10  => (factor * v / 10.0),
        V0To5         |
        VMinus5To5    => (factor * v / 5.0),
        V1To5         => (factor * (v - 1.0) / 4.0),
        V2To10        => (factor * (v - 2.0) / 8.0),
        Disabled      => 0.0,
    };
    v as u16
}

pub fn u16_to_analog_ui_value(
    data: u16,
    range: &AnalogUIRange,
    format: &DataFormat,
) -> Option<f32> {
    let factor = format.factor();
    use crate::AnalogUIRange::*;
    let data = f32::from(data as i16);

    #[cfg_attr(rustfmt, rustfmt_skip)]
    match *range {
        mA0To20         => Some(data * 20.0 / factor),
        mA4To20         => Some(data * 16.0 / factor + 4.0),
        V0To10          |
        VMinus10To10    => Some(data * 10.0 / factor),
        V0To5           |
        VMinus5To5      => Some(data * 5.0 / factor),
        V1To5           => Some(data * 4.0 / factor + 1.0),
        V2To10          => Some(data * 8.0 / factor + 2.0),
        Disabled        => None,
    }
}

pub fn u16_to_rtd_value(data: u16, range: &RtdRange) -> Option<f32> {
    use crate::RtdRange::*;

    #[cfg_attr(rustfmt, rustfmt_skip)]
    match *range {
        PT100  |
        PT200  |
        PT500  |
        PT1000 |
        NI100  |
        NI120  |
        NI200  |
        NI500  |
        NI1000 |
        Cu10   => {
            Some(f32::from(data as i16) / 10.0)
        }
        R40   |
        R80   |
        R150  |
        R300  |
        R500  |
        R1000 |
        R2000 |
        R4000 => {
            let n = match *range {
                R40   => 40.0,
                R80   => 80.0,
                R150  => 150.0,
                R300  => 300.0,
                R500  => 500.0,
                R1000 => 1000.0,
                R2000 => 2000.0,
                R4000 => 4000.0,
                _ => {
                    unreachable!()
                }
            };
            let d = n * u32::from(data) as f32 / 0x6C00 as f32;
            Some(d)
        }
        Disabled => None
    }
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

    #[test]
    fn shift_data() {
        assert_eq!(super::shift_data(&vec![0xABCD]), vec![0x00AB]);
    }

    #[test]
    fn test_u16_to_analog_ui_value() {
        use super::*;
        assert_eq!(
            u16_to_analog_ui_value(0x3600, &AnalogUIRange::mA0To20, &DataFormat::S7),
            Some(10.0)
        );
        assert_eq!(
            u16_to_analog_ui_value(0x2000, &AnalogUIRange::mA0To20, &DataFormat::S5),
            Some(10.0)
        );

        assert_eq!(
            u16_to_analog_ui_value(0x3600, &AnalogUIRange::mA4To20, &DataFormat::S7),
            Some(12.0)
        );
        assert_eq!(
            u16_to_analog_ui_value(0x2000, &AnalogUIRange::mA4To20, &DataFormat::S5),
            Some(12.0)
        );

        assert_eq!(
            u16_to_analog_ui_value(0x3600, &AnalogUIRange::V0To10, &DataFormat::S7),
            Some(5.0)
        );
        assert_eq!(
            u16_to_analog_ui_value(0x2000, &AnalogUIRange::V0To10, &DataFormat::S5),
            Some(5.0)
        );

        assert_eq!(
            u16_to_analog_ui_value(0x3600, &AnalogUIRange::VMinus10To10, &DataFormat::S7),
            Some(5.0)
        );
        assert_eq!(
            u16_to_analog_ui_value(0x2000, &AnalogUIRange::VMinus10To10, &DataFormat::S5),
            Some(5.0)
        );

        assert_eq!(
            u16_to_analog_ui_value(0x3600, &AnalogUIRange::V2To10, &DataFormat::S7),
            Some(6.0)
        );
        assert_eq!(
            u16_to_analog_ui_value(0x2000, &AnalogUIRange::V2To10, &DataFormat::S5),
            Some(6.0)
        );

        assert_eq!(
            u16_to_analog_ui_value(0x3600, &AnalogUIRange::V1To5, &DataFormat::S7),
            Some(3.0)
        );
        assert_eq!(
            u16_to_analog_ui_value(0x2000, &AnalogUIRange::V1To5, &DataFormat::S5),
            Some(3.0)
        );

        assert_eq!(
            u16_to_analog_ui_value(0x3600, &AnalogUIRange::V0To5, &DataFormat::S7),
            Some(2.5)
        );
        assert_eq!(
            u16_to_analog_ui_value(0x2000, &AnalogUIRange::V0To5, &DataFormat::S5),
            Some(2.5)
        );

        assert_eq!(
            u16_to_analog_ui_value(0x3600, &AnalogUIRange::VMinus5To5, &DataFormat::S7),
            Some(2.5)
        );
        assert_eq!(
            u16_to_analog_ui_value(0xCA00, &AnalogUIRange::VMinus5To5, &DataFormat::S7),
            Some(-2.5)
        );
        assert_eq!(
            u16_to_analog_ui_value(0x2000, &AnalogUIRange::VMinus5To5, &DataFormat::S5),
            Some(2.5)
        );
        assert_eq!(
            u16_to_analog_ui_value(0xE000, &AnalogUIRange::VMinus5To5, &DataFormat::S5),
            Some(-2.5)
        );
        assert_eq!(
            u16_to_analog_ui_value(0xE000, &AnalogUIRange::Disabled, &DataFormat::S5),
            None
        );
    }

    #[test]
    fn test_analog_ui_value_to_u16() {
        use super::*;
        assert_eq!(
            analog_ui_value_to_u16(10.0, &AnalogUIRange::mA0To20, &DataFormat::S7),
            0x3600
        );
    }
}
