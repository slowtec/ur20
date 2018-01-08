//! Digital output module UR20-4DO-P

use super::*;
use util::*;

#[derive(Debug)]
pub struct Mod {
    pub ch_params: Vec<ChannelParameters>,
}

#[derive(Debug, Clone)]
pub struct ChannelParameters {
    pub substitute_value: bool,
}

impl Default for ChannelParameters {
    fn default() -> Self {
        ChannelParameters { substitute_value: false }
    }
}

impl Default for Mod {
    fn default() -> Self {
        let ch_params = (0..4).map(|_| ChannelParameters::default()).collect();
        Mod { ch_params }
    }
}

impl Module for Mod {
    fn process_input_byte_count(&self) -> usize {
        0
    }
    fn process_output_byte_count(&self) -> usize {
        1
    }
    fn module_type(&self) -> ModuleType {
        ModuleType::UR20_4DI_P
    }
    fn process_input(&mut self, _: &[u16]) -> Result<Vec<ChannelValue>> {
        Ok((0..4).map(|_| ChannelValue::None).collect())
    }
    fn values_into_output_data(&mut self, values: &[ChannelValue]) -> Result<Vec<u16>> {
        if values.len() != 4 {
            return Err(Error::ChannelValue);
        }
        let mut res = 0;
        for (i, v) in values.into_iter().enumerate() {
            match *v {
                ChannelValue::Bit(state) => {
                    if state {
                        res = set_bit_16(res, i);
                    }
                }
                _ => {
                    return Err(Error::ChannelValue);
                }
            }
        }
        Ok(vec![res])
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use ChannelValue::*;

    #[test]
    fn test_values_into_output_data_with_invalid_channel_len() {
        let mut m = Mod::default();
        assert!(m.values_into_output_data(&[]).is_err());
        assert!(
            m.values_into_output_data(&[Bit(true), Bit(false), Bit(true)])
                .is_err()
        );
        assert!(
            m.values_into_output_data(&[Bit(true), Bit(false), Bit(true), Bit(true)])
                .is_ok()
        );
    }

    #[test]
    fn test_values_into_output_data_with_invalid_channel_values() {
        let mut m = Mod::default();
        assert!(
            m.values_into_output_data(&[Bit(false), Bit(true), Bit(false), Decimal32(0.0)])
                .is_err()
        );
    }

    #[test]
    fn test_values_into_output_data() {
        let mut m = Mod::default();
        assert_eq!(
            m.values_into_output_data(&[Bit(true), Bit(false), Bit(true), Bit(true)])
                .unwrap(),
            vec![0b0000_0000_0000_0000_1101]
        );
    }
}
