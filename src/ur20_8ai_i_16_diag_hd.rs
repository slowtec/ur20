//! Analog input module UR20-8AI-I-16-DIAG-HD

use super::*;

#[derive(Debug, Clone)]
pub struct ModuleParameters {
    pub frequency_suppression: FrequencySuppression
}

#[derive(Debug, Clone)]
pub struct ChannelParameters {
    pub channel_diagnostics: bool,
    pub diag_short_circuit: bool,
    pub data_format: DataFormat,
    pub measurement_range: AnalogIRange,
}
