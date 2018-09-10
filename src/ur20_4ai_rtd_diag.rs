//! Analog input module UR20-4AI-RTD-DIAG

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
    pub temperature_unit: TemperatureUnit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelParameters {
    pub measurement_range: RtdRange,
    pub connection_type: ConnectionType,
    pub conversion_time: ConversionTime,
    pub channel_diagnostics: bool,
    pub limit_value_monitoring: bool,
    //-32768 ... 32767
    pub high_limit_value: i16,
    //-32768 ... 32767
    pub low_limit_value: i16,
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
            temperature_unit: TemperatureUnit::Celsius,
        }
    }
}

impl Default for ChannelParameters {
    fn default() -> Self {
        ChannelParameters {
            measurement_range: RtdRange::Disabled,
            connection_type: ConnectionType::TwoWire,
            conversion_time: ConversionTime::ms80,
            channel_diagnostics: false,
            limit_value_monitoring: false,
            high_limit_value: 0,
            low_limit_value: 0,
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
        ModuleType::UR20_4AI_RTD_DIAG
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
            .map(|i| (data[i], &self.ch_params[i].measurement_range))
            .map(|(val, range)| match util::u16_to_rtd_value(val, range) {
                Some(v) => ChannelValue::Decimal32(v),
                None => ChannelValue::Disabled,
            })
            .collect();
        Ok(res)
    }
}

fn parameters_from_raw_data(data: &[u16]) -> Result<(ModuleParameters, Vec<ChannelParameters>)> {
    if data.len() < 29 {
        return Err(Error::BufferLength);
    }
    let mut module_parameters = ModuleParameters::default();

    module_parameters.temperature_unit = match FromPrimitive::from_u16(data[0]) {
        Some(x) => x,
        _ => {
            return Err(Error::ChannelParameter);
        }
    };

    let channel_parameters: Result<Vec<_>> = (0..4)
        .map(|i| {
            let mut p = ChannelParameters::default();
            let idx = i * 7;

            p.measurement_range = match FromPrimitive::from_u16(data[idx + 1]) {
                Some(x) => x,
                _ => {
                    return Err(Error::ChannelParameter);
                }
            };

            p.connection_type = match FromPrimitive::from_u16(data[idx + 2]) {
                Some(x) => x,
                _ => {
                    return Err(Error::ChannelParameter);
                }
            };

            p.conversion_time = match FromPrimitive::from_u16(data[idx + 3]) {
                Some(x) => x,
                _ => {
                    return Err(Error::ChannelParameter);
                }
            };

            p.channel_diagnostics = match data[idx + 4] {
                0 => false,
                1 => true,
                _ => {
                    return Err(Error::ChannelParameter);
                }
            };

            p.limit_value_monitoring = match data[idx + 5] {
                0 => false,
                1 => true,
                _ => {
                    return Err(Error::ChannelParameter);
                }
            };

            p.high_limit_value = data[idx + 6] as i16;
            p.low_limit_value = data[idx + 7] as i16;

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
        assert!(m.process_input_data(&vec![0, 0, 0, 0]).is_err());
    }

    #[test]
    fn test_process_input_data_with_disabled_channels() {
        let m = Mod::default();
        assert_eq!(
            m.process_input_data(&vec![5, 0, 7, 8]).unwrap(),
            vec![Disabled, Disabled, Disabled, Disabled]
        );
    }

    #[test]
    fn test_process_input_data() {
        let mut m = Mod::default();

        m.ch_params[0].measurement_range = RtdRange::R40;
        m.ch_params[1].measurement_range = RtdRange::R40;
        m.ch_params[2].measurement_range = RtdRange::PT100;
        m.ch_params[3].measurement_range = RtdRange::PT1000;

        assert_eq!(
            m.process_input_data(&vec![0x6C00, 0x7EFF, 55, 99]).unwrap(),
            vec![
                Decimal32(40.0),
                Decimal32(47.03559),
                Decimal32(5.5),
                Decimal32(9.9),
            ]
        );
    }

    #[test]
    fn test_process_input_data_with_negative_temperatures() {
        let mut m = Mod::default();
        m.ch_params[0].measurement_range = RtdRange::PT100;
        m.ch_params[1].measurement_range = RtdRange::Cu10;

        assert_eq!(
            m.process_input_data(&vec![0xF830, 0xFF38, 0, 0]).unwrap(),
            vec![Decimal32(-200.0), Decimal32(-20.0), Disabled, Disabled]
        );
    }

    #[test]
    fn test_process_input_data_with_underloading() {
        let mut m = Mod::default();

        m.ch_params[0].measurement_range = RtdRange::PT100;
        m.ch_params[1].measurement_range = RtdRange::NI1000;

        let input = m
            .process_input_data(&vec![(-2040_i16 as u16), (-640_i16 as u16), 0, 0])
            .unwrap();

        if let ChannelValue::Decimal32(v) = input[0] {
            assert_eq!(v, -204.0);
        } else {
            panic!();
        }

        if let ChannelValue::Decimal32(v) = input[1] {
            assert_eq!(v, -64.0);
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
        assert_eq!(
            m.process_output_values(&vec![ChannelValue::None; 4])
                .unwrap(),
            &[]
        );
    }

    #[test]
    fn test_module_parameters_from_raw_data() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let mut data = vec![
            0,                   // Module
            0, 0, 0, 0, 0, 0, 0, // CH 0
            0, 0, 0, 0, 0, 0, 0, // CH 1
            0, 0, 0, 0, 0, 0, 0, // CH 2
            0, 0, 0, 0, 0, 0, 0, // CH 3
        ];

        assert_eq!(
            parameters_from_raw_data(&data).unwrap().0.temperature_unit,
            TemperatureUnit::Celsius
        );
        data[0] = 1;
        assert_eq!(
            parameters_from_raw_data(&data).unwrap().0.temperature_unit,
            TemperatureUnit::Fahrenheit
        );
        data[0] = 2;
        assert_eq!(
            parameters_from_raw_data(&data).unwrap().0.temperature_unit,
            TemperatureUnit::Kelvin
        );
    }

    #[test]
    fn test_channel_parameters_from_raw_data() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let data = vec![
            0,                               // Module
            18, 0, 2, 0, 0, 0, 0,            // CH 0
            5,  1, 0, 0, 0, 0, 0,            // CH 1
            0,  0, 1, 0, 0, 0, 0,            // CH 2
            0,  0, 0, 1, 1, 0x7FFF, 0x8000,  // CH 3
        ];

        assert_eq!(parameters_from_raw_data(&data).unwrap().1.len(), 4);

        assert_eq!(
            parameters_from_raw_data(&data).unwrap().1[0],
            ChannelParameters::default()
        );

        assert_eq!(
            parameters_from_raw_data(&data).unwrap().1[1].measurement_range,
            RtdRange::NI120
        );

        assert_eq!(
            parameters_from_raw_data(&data).unwrap().1[1].connection_type,
            ConnectionType::ThreeWire
        );
        assert_eq!(
            parameters_from_raw_data(&data).unwrap().1[2].conversion_time,
            ConversionTime::ms130
        );

        assert_eq!(
            parameters_from_raw_data(&data).unwrap().1[3].channel_diagnostics,
            true
        );

        assert_eq!(
            parameters_from_raw_data(&data).unwrap().1[3].limit_value_monitoring,
            true
        );
        assert_eq!(
            parameters_from_raw_data(&data).unwrap().1[3].high_limit_value,
            ::std::i16::MAX
        );

        assert_eq!(
            parameters_from_raw_data(&data).unwrap().1[3].low_limit_value,
            ::std::i16::MIN
        );
    }

    #[test]
    fn test_parameters_from_invalid_raw_data() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let mut data = vec![
            0,                   // Module
            0, 0, 0, 0, 0, 0, 0, // CH 0
            0, 0, 0, 0, 0, 0, 0, // CH 1
            0, 0, 0, 0, 0, 0, 0, // CH 2
            0, 0, 0, 0, 0, 0, 0, // CH 3
        ];
        data[1] = 19; // should be max '18'
        assert!(parameters_from_raw_data(&data).is_err());

        data[1] = 0;
        data[2] = 3; // should be max '2'
        assert!(parameters_from_raw_data(&data).is_err());

        data[2] = 0;
        data[3] = 6; // should be max '5'
        assert!(parameters_from_raw_data(&data).is_err());
    }

    #[test]
    fn test_parameters_from_invalid_data_buffer_size() {
        let data = [0; 0];
        assert!(parameters_from_raw_data(&data).is_err());
        let data = [0; 28];
        assert!(parameters_from_raw_data(&data).is_err());
        let data = [0; 29];
        assert!(parameters_from_raw_data(&data).is_ok());
    }

    #[test]
    fn create_module_from_modbus_parameter_data() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let data = vec![
            0,                    // Module
            1,  0, 0, 0, 0, 0, 0, // CH 0
            18, 0, 0, 0, 0, 0, 0, // CH 1
            0,  0, 0, 0, 0, 0, 0, // CH 2
            0,  0, 0, 0, 0, 0, 0, // CH 3
        ];
        let module = Mod::from_modbus_parameter_data(&data).unwrap();
        assert_eq!(module.ch_params[0].measurement_range, RtdRange::PT200);
        assert_eq!(module.ch_params[1].measurement_range, RtdRange::Disabled);
    }
}
