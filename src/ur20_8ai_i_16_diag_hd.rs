//! Analog input module UR20-8AI-I-16-DIAG-HD

use super::*;

const S5_FACTOR: u16 = 16384;
const S7_FACTOR: u16 = 27648;

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
        ModuleParameters { frequency_suppression: FrequencySuppression::Disabled }
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
    fn process_input_word_count(&self) -> usize {
        8
    }

    fn process_input(&mut self, data: &[u16]) -> Result<Vec<ChannelValue>, Error> {

        use AnalogIRange::*;

        if data.len() != 8 {
            return Err(Error::BufferLength);
        }

        if self.ch_params.len() != 8 {
            return Err(Error::BufferLength);
        }

        let res = (0..8)
            .map(|i| {
                (
                    data[i] as u32,
                    &self.ch_params[i].measurement_range,
                    &self.ch_params[i].data_format,
                )
            })
            .map(|(val, range, format)| {
                let factor = match *format {
                    DataFormat::S5 => S5_FACTOR,
                    DataFormat::S7 => S7_FACTOR,
                } as f32;
                match *range {
                    mA0To20 => ChannelValue::Decimal32((val * 20) as f32 / factor),
                    mA4To20 => ChannelValue::Decimal32((val * 16) as f32 / factor + 4.0),
                    Disabled => ChannelValue::Disabled,
                }
            })
            .collect();
        Ok(res)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use ChannelValue::*;

    #[test]
    fn test_process_input_with_empty_buffer() {
        let mut m = Mod::default();
        assert!(m.process_input(&vec![]).is_err());
    }

    #[test]
    fn test_process_input_with_missing_channel_parameters() {
        let mut m = Mod::default();
        m.ch_params = vec![];
        assert!(m.process_input(&vec![0, 0, 0, 0, 0, 0, 0, 0]).is_err());
    }

    #[test]
    fn test_process_input() {
        let mut m = Mod::default();
        assert_eq!(
            m.process_input(&vec![5, 0, 7, 8, 0, 0, 0, 0]).unwrap(),
            vec![
                Disabled,
                Disabled,
                Disabled,
                Disabled,
                Disabled,
                Disabled,
                Disabled,
                Disabled,
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
            m.process_input(&vec![0x6C00, 0x3600, 0x4000, 0x6C00, 0x3600, 0x4000, 0, 0])
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
}
