//! Analog output module UR20-4AO-UI-16

use super::*;

#[derive(Debug, Clone)]
pub struct Parameters {
    pub data_format: DataFormat,
    pub output_range: AnalogUIRange,
    pub substitute_value: AnalogUIRange
}
