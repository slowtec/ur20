//! Analog input module UR20-4AI-UI-16-DIAG

use super::*;
use num_traits::cast::FromPrimitive;
use ur20_fbc_mod_tcp::{FromModbusParameterData, ProcessModbusTcpData};

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
    pub diag_line_break: bool,
    pub data_format: DataFormat,
    pub measurement_range: AnalogUIRange,
}

impl FromModbusParameterData for Mod {
    fn from_modbus_parameter_data(data: &[u16]) -> Result<Mod> {
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
            diag_line_break: false,
            data_format: DataFormat::S7,
            measurement_range: AnalogUIRange::Disabled,
        }
    }
}

impl Default for Mod {
    fn default() -> Self {
        let ch_params = (0..4).map(|_| ChannelParameters::default()).collect();
        let mod_params = ModuleParameters::default();
        Mod {
            mod_params,
            ch_params,
        }
    }
}

impl Module for Mod {
    fn module_type(&self) -> ModuleType {
        ModuleType::UR20_4AI_UI_16_DIAG
    }
}

impl ProcessModbusTcpData for Mod {
    fn process_input_byte_count(&self) -> usize {
        8
    }
    fn process_output_byte_count(&self) -> usize {
        0
    }
    fn process_input_data(&self, data: &[u16]) -> Result<Vec<ChannelValue>> {
        if data.len() != 4 {
            return Err(Error::BufferLength);
        }

        if self.ch_params.len() != 4 {
            return Err(Error::ChannelParameter);
        }

        let res = (0..4)
            .map(|i| {
                (
                    data[i],
                    &self.ch_params[i].measurement_range,
                    &self.ch_params[i].data_format,
                )
            })
            .map(
                |(val, range, format)| match util::u16_to_analog_ui_value(val, range, format) {
                    Some(v) => ChannelValue::Decimal32(v),
                    None => ChannelValue::Disabled,
                },
            )
            .collect();
        Ok(res)
    }
    fn process_output_data(&self, data: &[u16]) -> Result<Vec<ChannelValue>> {
        if !data.is_empty() {
            return Err(Error::BufferLength);
        }
        Ok((0..4).map(|_| ChannelValue::None).collect())
    }
    fn process_output_values(&self, values: &[ChannelValue]) -> Result<Vec<u16>> {
        if !values.is_empty() {
            //TODO: 4 x None should be ok
            return Err(Error::ChannelValue);
        }
        Ok(vec![])
    }
}

fn parameters_from_raw_data(data: &[u16]) -> Result<(ModuleParameters, Vec<ChannelParameters>)> {
    if data.len() < 21 {
        return Err(Error::BufferLength);
    }

    let frequency_suppression =
        FromPrimitive::from_u16(data[0]).ok_or_else(|| Error::ChannelParameter)?;

    let module_parameters = ModuleParameters {
        frequency_suppression,
    };

    let channel_parameters: Result<Vec<_>> = (0..4)
        .map(|i| {
            let mut p = ChannelParameters::default();
            let idx = i * 5;

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

            p.diag_line_break = match data[idx + 3] {
                0 => false,
                1 => true,
                _ => {
                    return Err(Error::ChannelParameter);
                }
            };

            p.data_format =
                FromPrimitive::from_u16(data[idx + 4]).ok_or_else(|| Error::ChannelParameter)?;

            p.measurement_range =
                FromPrimitive::from_u16(data[idx + 5]).ok_or_else(|| Error::ChannelParameter)?;

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
        assert!(m.process_input_data(&vec![0; 4]).is_err());
    }

    #[test]
    fn test_process_input_data() {
        let mut m = Mod::default();
        assert_eq!(m.ch_params[0].measurement_range, AnalogUIRange::Disabled);
        assert_eq!(m.ch_params[1].measurement_range, AnalogUIRange::Disabled);
        assert_eq!(m.ch_params[2].measurement_range, AnalogUIRange::Disabled);
        assert_eq!(m.ch_params[3].measurement_range, AnalogUIRange::Disabled);
        assert_eq!(
            m.process_input_data(&vec![5, 0, 7, 8]).unwrap(),
            vec![Disabled; 4]
        );

        m.ch_params[0].measurement_range = AnalogUIRange::mA0To20;
        m.ch_params[1].measurement_range = AnalogUIRange::VMinus5To5;
        m.ch_params[2].measurement_range = AnalogUIRange::V2To10;
        m.ch_params[3].measurement_range = AnalogUIRange::V0To5;

        m.ch_params[2].data_format = DataFormat::S5;

        assert_eq!(
            m.process_input_data(&vec![0x6C00, 0x3600, 0x4000, 0x6C00])
                .unwrap(),
            vec![
                Decimal32(20.0),
                Decimal32(2.5),
                Decimal32(10.0),
                Decimal32(5.0),
            ]
        );
    }

    #[test]
    fn test_process_input_data_with_underloading() {
        let mut m = Mod::default();

        m.ch_params[0].measurement_range = AnalogUIRange::mA4To20;
        m.ch_params[0].data_format = DataFormat::S7;

        m.ch_params[1].measurement_range = AnalogUIRange::mA4To20;
        m.ch_params[1].data_format = DataFormat::S5;

        let input = m.process_input_data(&vec![0xED00, 0x0F333, 0, 0]).unwrap();

        if let ChannelValue::Decimal32(v) = input[0] {
            assert!((v - 1.19).abs() < 0.01);
        } else {
            panic!();
        }
        if let ChannelValue::Decimal32(v) = input[1] {
            assert!((v - 0.8).abs() < 0.01);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_process_output_data() {
        let m = Mod::default();
        assert!(m.process_output_data(&vec![0; 4]).is_err());
        assert_eq!(
            m.process_output_data(&[]).unwrap(),
            vec![ChannelValue::None; 4]
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
        let mut data = vec![0; 21];
        assert_eq!(
            parameters_from_raw_data(&data)
                .unwrap()
                .0
                .frequency_suppression,
            FrequencySuppression::Disabled
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
            0,             // Module
            0, 0, 0, 1, 8, // CH 0
            1, 0, 0, 0, 5, // CH 1
            0, 1, 0, 0, 0, // CH 2
            0, 0, 1, 0, 0, // CH 3
        ];

        assert_eq!(parameters_from_raw_data(&data).unwrap().1.len(), 4);

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
            parameters_from_raw_data(&data).unwrap().1[1].diag_line_break,
            false
        );

        assert_eq!(
            parameters_from_raw_data(&data).unwrap().1[1].data_format,
            DataFormat::S5
        );

        assert_eq!(
            parameters_from_raw_data(&data).unwrap().1[1].measurement_range,
            AnalogUIRange::VMinus5To5
        );

        assert_eq!(
            parameters_from_raw_data(&data).unwrap().1[2].diag_short_circuit,
            true
        );
        assert_eq!(
            parameters_from_raw_data(&data).unwrap().1[3].diag_line_break,
            true
        );
        assert_eq!(
            parameters_from_raw_data(&data).unwrap().1[3].measurement_range,
            AnalogUIRange::mA0To20
        );
    }

    #[test]
    fn test_parameters_from_invalid_raw_data() {
        let mut data = vec![0; 21];

        data[0] = 4; // should be max '3'
        assert!(parameters_from_raw_data(&data).is_err());

        data[0] = 0;
        data[1] = 2; // should be '0' or '1'
        assert!(parameters_from_raw_data(&data).is_err());

        data[1] = 0;
        data[2] = 2; // should be '0' or '1'
        assert!(parameters_from_raw_data(&data).is_err());

        data[2] = 0;
        data[3] = 2; // should be '0' or '1'
        assert!(parameters_from_raw_data(&data).is_err());

        data[3] = 0;
        data[4] = 2; // should be '0' or '1'
        assert!(parameters_from_raw_data(&data).is_err());

        data[4] = 0;
        data[5] = 9; // should be max '8'
        assert!(parameters_from_raw_data(&data).is_err());
    }

    #[test]
    fn test_parameters_from_invalid_data_buffer_size() {
        let data = [0; 0];
        assert!(parameters_from_raw_data(&data).is_err());
        let data = [0; 20];
        assert!(parameters_from_raw_data(&data).is_err());
        let data = [0; 21];
        assert!(parameters_from_raw_data(&data).is_ok());
    }

    #[test]
    fn create_module_from_modbus_parameter_data() {
        let data = vec![
            0,             // Module
            0, 0, 0, 0, 1, // CH 0
            0, 0, 0, 1, 8, // CH 1
            1, 0, 0, 0, 0, // CH 2
            0, 0, 0, 0, 0, // CH 3
        ];
        let module = Mod::from_modbus_parameter_data(&data).unwrap();
        assert_eq!(
            module.ch_params[0].measurement_range,
            AnalogUIRange::mA4To20
        );
        assert_eq!(
            module.ch_params[1].measurement_range,
            AnalogUIRange::Disabled
        );
        assert_eq!(module.ch_params[2].channel_diagnostics, true);
    }
}
