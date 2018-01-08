//! Digital output module UR20-4DO-P

use super::*;

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
    fn module_type(&self) -> ModuleType {
        ModuleType::UR20_4DI_P
    }
    fn process_input(&mut self, _: &[u16]) -> Result<Vec<ChannelValue>, Error> {
        Ok((0..4).map(|_| ChannelValue::None).collect())

    }
}
