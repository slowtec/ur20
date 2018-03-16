//! Analog input module UR20-8AI-I-16-DIAG-HD

use super::*;
use num_traits::cast::FromPrimitive;

#[derive(Debug)]
pub struct Mod {
    pub mod_params: ModuleParameters,
    pub ch_params: Vec<ChannelParameters>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleParameters {
    pub frequency_suppression: FrequencySuppression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelParameters {
    pub channel_diagnostics: bool,
    pub diag_short_circuit: bool,
    pub data_format: DataFormat,
    pub measurement_range: AnalogIRange,
}

impl Mod {
    pub fn from_parameter_data(data: &[u16]) -> Result<Mod> {
        let (mod_params, ch_params) = parameters_from_raw_data(data)?;
        Ok(Mod {
            mod_params,
            ch_params,
        })
    }
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

fn parameters_from_raw_data(data: &[u16]) -> Result<(ModuleParameters, Vec<ChannelParameters>)> {
    if data.len() < 33 {
        return Err(Error::BufferLength);
    }
    let mut module_parameters = ModuleParameters::default();

    module_parameters.frequency_suppression = match data[0] {
        0 => FrequencySuppression::Disabled,
        1 => FrequencySuppression::Hz50,
        2 => FrequencySuppression::Hz60,
        3 => FrequencySuppression::Average16,
        _ => return Err(Error::ChannelParameter),
    };

    let channel_parameters: Result<Vec<_>> = (0..8)
        .map(|i| {
            let mut p = ChannelParameters::default();
            let idx = i * 4;

            p.channel_diagnostics = match data[idx + 1] {
                0 => false,
                1 => true,
                _ => {
                    return Err(Error::ChannelParameter);
                }
            };

            p.diag_short_circuit = match data[idx + 2] {
                0 => false,
                1 => true,
                _ => {
                    return Err(Error::ChannelParameter);
                }
            };

            p.data_format = match FromPrimitive::from_u16(data[idx + 3]) {
                Some(f) => f,
                _ => {
                    return Err(Error::ChannelParameter);
                }
            };

            p.measurement_range = match FromPrimitive::from_u16(data[idx + 4]) {
                Some(r) => r,
                _ => {
                    return Err(Error::ChannelParameter);
                }
            };
            Ok(p)
        })
        .collect();
    Ok((module_parameters, channel_parameters?))
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

    #[test]
    fn test_module_parameters_from_raw_data() {
        let mut data = vec![
            0,          // Module
            0, 0, 0, 0, // CH 0
            0, 0, 0, 0, // CH 1
            0, 0, 0, 0, // CH 2
            0, 0, 0, 0, // CH 3
            0, 0, 0, 0, // CH 4
            0, 0, 0, 0, // CH 5
            0, 0, 0, 0, // CH 6
            0, 0, 0, 0, // CH 7
        ];

        assert_eq!(
            parameters_from_raw_data(&data)
                .unwrap()
                .0
                .frequency_suppression,
            FrequencySuppression::Disabled
        );
        data[0] = 1;
        assert_eq!(
            parameters_from_raw_data(&data)
                .unwrap()
                .0
                .frequency_suppression,
            FrequencySuppression::Hz50
        );
        data[0] = 2;
        assert_eq!(
            parameters_from_raw_data(&data)
                .unwrap()
                .0
                .frequency_suppression,
            FrequencySuppression::Hz60
        );
        data[0] = 3;
        assert_eq!(
            parameters_from_raw_data(&data)
                .unwrap()
                .0
                .frequency_suppression,
            FrequencySuppression::Average16
        );
    }

    #[test]
    fn test_channel_parameters_from_raw_data() {
        let data = vec![
            0,          // Module
            0, 0, 1, 2, // CH 0
            1, 0, 0, 2, // CH 1
            0, 1, 0, 2, // CH 2
            0, 0, 1, 2, // CH 3
            0, 0, 0, 1, // CH 4
            1, 1, 1, 0, // CH 5
            0, 0, 0, 0, // CH 6
            0, 0, 0, 0, // CH 7
        ];

        assert_eq!(parameters_from_raw_data(&data).unwrap().1.len(), 8);
        assert_eq!(
            parameters_from_raw_data(&data).unwrap().1[0],
            ChannelParameters::default()
        );

        assert_eq!(
            parameters_from_raw_data(&data).unwrap().1[1].channel_diagnostics,
            true
        );
        assert_eq!(
            parameters_from_raw_data(&data).unwrap().1[1].diag_short_circuit,
            false
        );
        assert_eq!(
            parameters_from_raw_data(&data).unwrap().1[1].data_format,
            DataFormat::S5
        );
        assert_eq!(
            parameters_from_raw_data(&data).unwrap().1[1].measurement_range,
            AnalogIRange::Disabled
        );

        assert_eq!(
            parameters_from_raw_data(&data).unwrap().1[2].diag_short_circuit,
            true
        );
        assert_eq!(
            parameters_from_raw_data(&data).unwrap().1[3].data_format,
            DataFormat::S7
        );
        assert_eq!(
            parameters_from_raw_data(&data).unwrap().1[4].measurement_range,
            AnalogIRange::mA4To20
        );
    }

    #[test]
    fn test_parameters_from_invalid_raw_data() {
        let mut data = vec![
            0,          // Module
            0, 0, 0, 0, // CH 0
            0, 0, 0, 0, // CH 1
            0, 0, 0, 0, // CH 2
            0, 0, 0, 0, // CH 3
            0, 0, 0, 0, // CH 4
            0, 0, 0, 0, // CH 5
            0, 0, 0, 0, // CH 6
            0, 0, 0, 0, // CH 7
        ];
        data[1] = 2; // should be '0' or '1'
        assert!(parameters_from_raw_data(&data).is_err());

        data[1] = 0;
        data[2] = 2; // should be '0' or '1'
        assert!(parameters_from_raw_data(&data).is_err());

        data[2] = 0;
        data[3] = 2; // should be '0' or '1'
        assert!(parameters_from_raw_data(&data).is_err());

        data[3] = 0;
        data[4] = 3; // should be '0','1' or '2'
        assert!(parameters_from_raw_data(&data).is_err());
    }

    #[test]
    fn test_parameters_from_invalid_data_buffer_size() {
        let data = [0; 0];
        assert!(parameters_from_raw_data(&data).is_err());
        let data = [0; 32];
        assert!(parameters_from_raw_data(&data).is_err());
        let data = [0; 33];
        assert!(parameters_from_raw_data(&data).is_ok());
    }

    #[test]
    fn create_module_from_parameter_data() {
        let data = vec![
            0,          // Module
            0, 0, 0, 0, // CH 0
            0, 0, 0, 2, // CH 1
            1, 0, 0, 0, // CH 2
            0, 0, 0, 0, // CH 3
            0, 0, 0, 0, // CH 4
            0, 0, 0, 0, // CH 5
            0, 0, 0, 0, // CH 6
            0, 0, 0, 0, // CH 7
        ];
        let module = Mod::from_parameter_data(&data).unwrap();
        assert_eq!(module.ch_params[0].measurement_range, AnalogIRange::mA0To20);
        assert_eq!(
            module.ch_params[1].measurement_range,
            AnalogIRange::Disabled
        );
        assert_eq!(module.ch_params[2].channel_diagnostics, true);
    }
}
