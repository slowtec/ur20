//! Digital input module UR20-4DI-P

use super::*;
use super::util::test_bit_16;

pub struct Mod {
    pub ch_params: Vec<ChannelParameters>
}

#[derive(Debug, Clone)]
pub struct ChannelParameters {
    pub input_delay: InputDelay,
}

impl Default for ChannelParameters {
    fn default() -> Self {
        ChannelParameters {
            input_delay: InputDelay::ms3,
        }
    }
}

impl Default for Mod {
    fn default() -> Self {
        let ch_params = (0..4).map(|_| ChannelParameters::default()).collect();
        Mod { ch_params }
    }
}

impl Module for Mod {
    fn process_input_word_count(&self) -> usize {
        4
    }

    fn process_input(&mut self, data: &[u16]) -> Result<Vec<ChannelValue>, Error> {
        if data.len() != 1 {
            return Err(Error::BufferLength);
        }
        let bits = data[0];
        let res = (0..4)
            .map(|i| ChannelValue::Bit(test_bit_16(bits,i)))
            .collect();
        Ok(res)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use ChannelValue::*;

    #[test]
    fn test_process_input() {
        let mut m = Mod::default();
        assert!(m.process_input(&vec![]).is_err());
        let data = vec![0b0100];
        assert_eq!(
            m.process_input(&data).unwrap(),
        vec![
            Bit(false),
            Bit(false),
            Bit(true),
            Bit(false)
        ]
        );
    }
}
