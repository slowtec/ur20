//! Digital input module UR20-4DI-P

use super::*;
use super::util::test_bit_16;

#[derive(Debug)]
pub struct Mod {
    pub ch_params: Vec<ChannelParameters>,
}

#[derive(Debug, Clone)]
pub struct ChannelParameters {
    pub input_delay: InputDelay,
}

impl Default for ChannelParameters {
    fn default() -> Self {
        ChannelParameters { input_delay: InputDelay::ms3 }
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
        1
    }
    fn process_output_byte_count(&self) -> usize {
        0
    }
    fn module_type(&self) -> ModuleType {
        ModuleType::UR20_4DI_P
    }
    fn process_input_data(&mut self, data: &[u16]) -> Result<Vec<ChannelValue>> {
        if data.len() != 1 {
            return Err(Error::BufferLength);
        }
        let bits = data[0];
        let res = (0..4)
            .map(|i| ChannelValue::Bit(test_bit_16(bits, i)))
            .collect();
        Ok(res)
    }
    fn process_output_values(&mut self, values: &[ChannelValue]) -> Result<Vec<u16>> {
        if values.len() != 0 {
            return Err(Error::ChannelValue);
        }
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use ChannelValue::*;

    #[test]
    fn test_process_input_data() {
        let mut m = Mod::default();
        assert!(m.process_input_data(&vec![]).is_err());
        let data = vec![0b0100];
        assert_eq!(
            m.process_input_data(&data).unwrap(),
            vec![Bit(false), Bit(false), Bit(true), Bit(false)]
        );
    }

    #[test]
    fn test_process_output_values() {
        let mut m = Mod::default();
        assert!(
            m.process_output_values(&[ChannelValue::Bit(true)])
                .is_err()
        );
        assert_eq!(m.process_output_values(&[]).unwrap(), &[]);
    }
}
