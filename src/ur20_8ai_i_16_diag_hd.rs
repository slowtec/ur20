//! Analog input module UR20-8AI-I-16-DIAG-HD

use super::*;

#[derive(Debug)]
pub struct Mod {
    pub mod_params: ModuleParameters,
    pub ch_params: Vec<ChannelParameters>,
}

#[derive(Debug, Clone)]
pub struct ModuleParameters {
    pub frequency_suppression: FrequencySuppression,
}

#[derive(Debug, Clone)]
pub struct ChannelParameters {
    pub channel_diagnostics: bool,
    pub diag_short_circuit: bool,
    pub data_format: DataFormat,
    pub measurement_range: AnalogIRange,
}

impl Default for ModuleParameters {
    fn default() -> Self {
        ModuleParameters {
            frequency_suppression: FrequencySuppression::Disabled,
        }
    }
}

impl Default for ChannelParameters {
    fn default() -> Self {
        ChannelParameters {
            channel_diagnostics: false,
            diag_short_circuit: false,
            data_format: DataFormat::S7,
            measurement_range: AnalogIRange::Disabled,
        }
    }
}

impl Default for Mod {
    fn default() -> Self {
        let ch_params = (0..8).map(|_| ChannelParameters::default()).collect();

        let mod_params = ModuleParameters::default();

        Mod {
            mod_params,
            ch_params,
        }
    }
}

impl Module for Mod {
    fn process_input_byte_count(&self) -> usize {
        16
    }
    fn process_output_byte_count(&self) -> usize {
        0
    }
    fn module_type(&self) -> ModuleType {
        ModuleType::UR20_8AI_I_16_DIAG_HD
    }
    fn process_input_data(&self, data: &[u16]) -> Result<Vec<ChannelValue>> {
        use AnalogIRange::*;

        if data.len() != 8 {
            return Err(Error::BufferLength);
        }

        if self.ch_params.len() != 8 {
            return Err(Error::ChannelParameter);
        }

        let res = (0..8)
            .map(|i| {
                (
                    u32::from(data[i]),
                    &self.ch_params[i].measurement_range,
                    &self.ch_params[i].data_format,
                )
            })
            .map(|(val, range, format)| {
                let factor = f32::from(match *format {
                    DataFormat::S5 => S5_FACTOR,
                    DataFormat::S7 => S7_FACTOR,
                });
                match *range {
                    mA0To20 => ChannelValue::Decimal32((val * 20) as f32 / factor),
                    mA4To20 => ChannelValue::Decimal32((val * 16) as f32 / factor + 4.0),
                    Disabled => ChannelValue::Disabled,
                }
            })
            .collect();
        Ok(res)
    }
    fn process_output_values(&self, values: &[ChannelValue]) -> Result<Vec<u16>> {
        if !values.is_empty() {
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
    fn test_process_input_data_with_empty_buffer() {
        let m = Mod::default();
        assert!(m.process_input_data(&vec![]).is_err());
    }

    #[test]
    fn test_process_input_data_with_missing_channel_parameters() {
        let mut m = Mod::default();
        m.ch_params = vec![];
        assert!(m.process_input_data(&vec![0, 0, 0, 0, 0, 0, 0, 0]).is_err());
    }

    #[test]
    fn test_process_input_data() {
        let mut m = Mod::default();
        assert_eq!(
            m.process_input_data(&vec![5, 0, 7, 8, 0, 0, 0, 0]).unwrap(),
            vec![
                Disabled, Disabled, Disabled, Disabled, Disabled, Disabled, Disabled, Disabled
            ]
        );

        m.ch_params[0].measurement_range = AnalogIRange::mA0To20;
        m.ch_params[1].measurement_range = AnalogIRange::mA0To20;
        m.ch_params[2].measurement_range = AnalogIRange::mA0To20;
        m.ch_params[2].data_format = DataFormat::S5;

        m.ch_params[3].measurement_range = AnalogIRange::mA4To20;
        m.ch_params[4].measurement_range = AnalogIRange::mA4To20;
        m.ch_params[5].measurement_range = AnalogIRange::mA4To20;
        m.ch_params[5].data_format = DataFormat::S5;

        assert_eq!(
            m.process_input_data(&vec![0x6C00, 0x3600, 0x4000, 0x6C00, 0x3600, 0x4000, 0, 0])
                .unwrap(),
            vec![
                Decimal32(20.0),
                Decimal32(10.0),
                Decimal32(20.0),
                Decimal32(20.0),
                Decimal32(12.0),
                Decimal32(20.0),
                Disabled,
                Disabled,
            ]
        );
    }

    #[test]
    fn test_process_output_values() {
        let m = Mod::default();
        assert!(
            m.process_output_values(&[ChannelValue::Decimal32(0.0)])
                .is_err()
        );
        assert_eq!(m.process_output_values(&[]).unwrap(), &[]);
    }
}
