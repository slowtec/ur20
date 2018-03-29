//! Analog output module UR20-4AO-UI-16-DIAG

use super::*;
use num_traits::cast::FromPrimitive;
use ur20_fbc_mod_tcp::{FromModbusParameterData, ProcessModbusTcpData};

#[derive(Debug)]
pub struct Mod {
    pub ch_params: Vec<ChannelParameters>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelParameters {
    pub data_format: DataFormat,
    pub output_range: AnalogUIRange,
    pub substitute_value: f32,
    pub channel_diagnostics: bool,
}

impl FromModbusParameterData for Mod {
    fn from_modbus_parameter_data(data: &[u16]) -> Result<Mod> {
        let ch_params = parameters_from_raw_data(data)?;
        Ok(Mod { ch_params })
    }
}

impl Default for ChannelParameters {
    fn default() -> Self {
        ChannelParameters {
            data_format: DataFormat::S7,
            output_range: AnalogUIRange::Disabled,
            substitute_value: 0.0,
            channel_diagnostics: false,
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
    fn module_type(&self) -> ModuleType {
        ModuleType::UR20_4AO_UI_16_DIAG
    }
}

impl ProcessModbusTcpData for Mod {
    fn process_input_byte_count(&self) -> usize {
        0
    }
    fn process_output_byte_count(&self) -> usize {
        8
    }
    fn process_input_data(&self, data: &[u16]) -> Result<Vec<ChannelValue>> {
        if !data.is_empty() {
            return Err(Error::BufferLength);
        }
        Ok((0..4).map(|_| ChannelValue::None).collect())
    }
    fn process_output_data(&self, data: &[u16]) -> Result<Vec<ChannelValue>> {
        if data.len() != 4 {
            return Err(Error::BufferLength);
        }
        Ok(data.into_iter()
            .enumerate()
            .map(|(i, v)| {
                (
                    v,
                    &self.ch_params[i].output_range,
                    &self.ch_params[i].data_format,
                )
            })
            .map(
                |(v, range, factor)| match util::u16_to_analog_ui_value(*v, range, factor) {
                    Some(v) => ChannelValue::Decimal32(v),
                    None => ChannelValue::Disabled,
                },
            )
            .collect())
    }
    fn process_output_values(&self, values: &[ChannelValue]) -> Result<Vec<u16>> {
        if values.len() != 4 {
            return Err(Error::ChannelValue);
        }
        if self.ch_params.len() != 4 {
            return Err(Error::ChannelParameter);
        }
        values
            .into_iter()
            .enumerate()
            .map(|(i, v)| {
                (
                    v,
                    &self.ch_params[i].output_range,
                    &self.ch_params[i].data_format,
                )
            })
            .map(|(v, range, factor)| value_to_u16(v, range, factor))
            .collect()
    }
}

fn value_to_u16(v: &ChannelValue, range: &AnalogUIRange, format: &DataFormat) -> Result<u16> {
    match *v {
        ChannelValue::Decimal32(v) => Ok(util::analog_ui_value_to_u16(v, range, format)),
        _ => Err(Error::ChannelValue),
    }
}

fn parameters_from_raw_data(data: &[u16]) -> Result<Vec<ChannelParameters>> {
    if data.len() < 16 {
        return Err(Error::BufferLength);
    }

    let channel_parameters: Result<Vec<_>> = (0..4)
        .map(|i| {
            let mut p = ChannelParameters::default();
            let idx = i * 4;

            p.data_format =
                FromPrimitive::from_u16(data[idx]).ok_or_else(|| Error::ChannelParameter)?;

            p.output_range =
                FromPrimitive::from_u16(data[idx + 1]).ok_or_else(|| Error::ChannelParameter)?;

            if let Some(v) =
                util::u16_to_analog_ui_value(data[idx + 2], &p.output_range, &p.data_format)
            {
                p.substitute_value = v;
            }
            p.channel_diagnostics = match data[idx + 3] {
                0 => false,
                1 => true,
                _ => {
                    return Err(Error::ChannelParameter);
                }
            };
            Ok(p)
        })
        .collect();
    Ok(channel_parameters?)
}

#[cfg(test)]
mod tests {

    use super::*;
    use ChannelValue::*;

    #[test]
    fn test_process_input_data() {
        let m = Mod::default();
        assert!(m.process_input_data(&[0, 0, 0, 0]).is_err());
        assert_eq!(
            m.process_input_data(&[]).unwrap(),
            &[
                ChannelValue::None,
                ChannelValue::None,
                ChannelValue::None,
                ChannelValue::None,
            ]
        );
    }

    #[test]
    fn test_process_output_data() {
        let mut m = Mod::default();
        assert_eq!(
            m.process_output_data(&vec![123, 456, 789, 0]).unwrap(),
            &[
                ChannelValue::Disabled,
                ChannelValue::Disabled,
                ChannelValue::Disabled,
                ChannelValue::Disabled,
            ]
        );
        m.ch_params[0].output_range = AnalogUIRange::mA0To20;
        m.ch_params[1].output_range = AnalogUIRange::mA0To20;
        m.ch_params[2].output_range = AnalogUIRange::mA0To20;
        m.ch_params[3].output_range = AnalogUIRange::mA0To20;
        assert_eq!(
            m.process_output_data(&vec![0x0, 0x6C00, 0x3600, 0x0])
                .unwrap(),
            &[
                Decimal32(0.0),
                Decimal32(20.0),
                Decimal32(10.0),
                Decimal32(0.0),
            ]
        );
    }

    #[test]
    fn test_process_output_data_with_invalid_buffer_size() {
        let m = Mod::default();
        assert!(m.process_output_data(&vec![]).is_err());
        assert!(m.process_output_data(&vec![0; 3]).is_err());
        assert!(m.process_output_data(&vec![0; 5]).is_err());
        assert!(m.process_output_data(&vec![0; 4]).is_ok());
    }

    #[test]
    fn test_process_output_values_with_invalid_channel_len() {
        let m = Mod::default();
        assert!(m.process_output_values(&[]).is_err());
        assert!(
            m.process_output_values(&[Decimal32(0.0), Decimal32(0.0), Decimal32(0.0)])
                .is_err()
        );
        assert!(m.process_output_values(&[
            Decimal32(0.0),
            Decimal32(0.0),
            Decimal32(0.0),
            Decimal32(0.0),
        ]).is_ok());
    }

    #[test]
    fn test_process_output_values_with_invalid_channel_values() {
        let m = Mod::default();
        assert!(m.process_output_values(&[
            Bit(false),
            Decimal32(0.0),
            Decimal32(0.0),
            Decimal32(0.0)
        ]).is_err());
    }

    #[test]
    fn test_process_output_values_with_missing_channel_parameters() {
        let mut m = Mod::default();
        m.ch_params = vec![];
        assert!(m.process_output_values(&[
            Decimal32(0.0),
            Decimal32(0.0),
            Decimal32(0.0),
            Decimal32(0.0),
        ]).is_err());
    }

    #[test]
    fn test_process_output_values() {
        let mut m = Mod::default();
        assert_eq!(
            m.process_output_values(&[
                Decimal32(0.0),
                Decimal32(99.9),
                Decimal32(0.0),
                Decimal32(3.3),
            ]).unwrap(),
            vec![0, 0, 0, 0]
        );
        m.ch_params[0].output_range = AnalogUIRange::mA0To20;
        m.ch_params[1].output_range = AnalogUIRange::mA0To20;
        m.ch_params[2].output_range = AnalogUIRange::mA0To20;
        m.ch_params[3].output_range = AnalogUIRange::mA0To20;
        assert_eq!(
            m.process_output_values(&[
                Decimal32(23.518),
                Decimal32(20.0),
                Decimal32(10.0),
                Decimal32(0.0),
            ]).unwrap(),
            vec![0x7EFF, 0x6C00, 0x3600, 0x0]
        );

        m.ch_params[0].output_range = AnalogUIRange::mA0To20;
        m.ch_params[1].output_range = AnalogUIRange::mA4To20;
        m.ch_params[2].output_range = AnalogUIRange::V0To10;
        m.ch_params[3].output_range = AnalogUIRange::VMinus10To10;
        assert_eq!(
            m.process_output_values(&[
                Decimal32(10.0),
                Decimal32(12.0),
                Decimal32(10.0),
                Decimal32(-10.0),
            ]).unwrap(),
            vec![0x3600, 0x3600, 0x6C00, 0x9400]
        );

        m.ch_params[0].output_range = AnalogUIRange::V0To5;
        m.ch_params[1].output_range = AnalogUIRange::VMinus5To5;
        m.ch_params[2].output_range = AnalogUIRange::V1To5;
        m.ch_params[3].output_range = AnalogUIRange::V2To10;
        assert_eq!(
            m.process_output_values(&[
                Decimal32(5.0),
                Decimal32(-5.0),
                Decimal32(1.0),
                Decimal32(2.0),
            ]).unwrap(),
            vec![0x6C00, 0x9400, 0x0, 0x0]
        );
    }

    #[test]
    fn test_channel_parameters_from_raw_data() {
        let data = vec![
            1, 8, 0,      0, // CH 0
            1, 0, 0,      1, // CH 1
            0, 2, 0,      0, // CH 2
            1, 5, 0xCA00, 0  // CH 3
        ];

        assert_eq!(parameters_from_raw_data(&data).unwrap().len(), 4);

        assert_eq!(
            parameters_from_raw_data(&data).unwrap()[0],
            ChannelParameters::default()
        );

        assert_eq!(
            parameters_from_raw_data(&data).unwrap()[1].data_format,
            DataFormat::S7
        );

        assert_eq!(
            parameters_from_raw_data(&data).unwrap()[1].channel_diagnostics,
            true
        );

        assert_eq!(
            parameters_from_raw_data(&data).unwrap()[2].output_range,
            AnalogUIRange::V0To10
        );

        assert_eq!(
            parameters_from_raw_data(&data).unwrap()[2].data_format,
            DataFormat::S5
        );

        assert_eq!(
            parameters_from_raw_data(&data).unwrap()[3].substitute_value,
            -2.5
        );
    }

    #[test]
    fn test_parameters_from_invalid_raw_data() {
        let mut data = vec![0; 16];

        data[0] = 2; // should be max '1'
        assert!(parameters_from_raw_data(&data).is_err());

        data[0] = 0;
        data[1] = 9; // should be max '8'
        assert!(parameters_from_raw_data(&data).is_err());

        data[1] = 0;
        data[3] = 2; // should be '0' or '1'
        assert!(parameters_from_raw_data(&data).is_err());
    }

    #[test]
    fn test_parameters_from_invalid_data_buffer_size() {
        let data = [0; 0];
        assert!(parameters_from_raw_data(&data).is_err());
        let data = [0; 15];
        assert!(parameters_from_raw_data(&data).is_err());
        let data = [0; 16];
        assert!(parameters_from_raw_data(&data).is_ok());
    }

    #[test]
    fn create_module_from_modbus_parameter_data() {
        let data = vec![
            1, 0, 0,0,  // CH 0
            0, 8, 0,0,  // CH 1
            0, 0, 0,0,  // CH 2
            0, 0, 0,0,  // CH 3
        ];
        let module = Mod::from_modbus_parameter_data(&data).unwrap();
        assert_eq!(module.ch_params[0].data_format, DataFormat::S7);
        assert_eq!(module.ch_params[1].output_range, AnalogUIRange::Disabled);
    }
}
