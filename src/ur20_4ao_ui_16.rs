//! Analog output module UR20-4AO-UI-16

use super::*;

#[derive(Debug)]
pub struct Mod {
    pub ch_params: Vec<ChannelParameters>,
}

#[derive(Debug, Clone)]
pub struct ChannelParameters {
    pub data_format: DataFormat,
    pub output_range: AnalogUIRange,
    pub substitute_value: f32,
}

impl Default for ChannelParameters {
    fn default() -> Self {
        ChannelParameters {
            data_format: DataFormat::S7,
            output_range: AnalogUIRange::Disabled,
            substitute_value: 0.0,
        }
    }
}

impl Module for Mod {
    fn process_input_byte_count(&self) -> usize {
        0
    }

    fn process_input(&mut self, _: &[u16]) -> Result<Vec<ChannelValue>, Error> {
        Ok((0..4).map(|_| ChannelValue::None).collect())
    }
}
