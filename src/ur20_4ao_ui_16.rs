//! Analog output module UR20-4AO-UI-16

use super::*;
use num_traits::cast::FromPrimitive;
use ur20_fbc_mod_tcp::ProcessModbusTcpData;

#[derive(Debug)]
pub struct Mod {
    pub ch_params: Vec<ChannelParameters>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelParameters {
    pub data_format: DataFormat,
    pub output_range: AnalogUIRange,
    pub substitute_value: f32,
}

impl Mod {
    pub fn from_parameter_data(data: &[u16]) -> Result<Mod> {
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
        ModuleType::UR20_4AO_UI_16
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
            .map(|(v, range, factor)| {
                if *range != AnalogUIRange::Disabled {
                    ChannelValue::Decimal32(u16_to_value(*v, range, factor))
                } else {
                    ChannelValue::Disabled
                }
            })
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
    let factor = f32::from(match *format {
        DataFormat::S5 => S5_FACTOR,
        DataFormat::S7 => S7_FACTOR,
    });
    match *v {
        ChannelValue::Decimal32(v) => {
            use AnalogUIRange::*;

            #[cfg_attr(rustfmt, rustfmt_skip)]
              Ok(match *range {
                  mA0To20       => (factor * v / 20.0),
                  mA4To20       => (factor * (v - 4.0) / 16.0),
                  V0To10        |
                  VMinus10To10  => (factor * v / 10.0),
                  V0To5         |
                  VMinus5To5    => (factor * v / 5.0),
                  V1To5         => (factor * (v - 1.0) / 4.0),
                  V2To10        => (factor * (v - 2.0) / 8.0),
                  Disabled      => 0.0,
              } as u16)
        }
        _ => Err(Error::ChannelValue),
    }
}

fn u16_to_value(data: u16, range: &AnalogUIRange, format: &DataFormat) -> f32 {
    let factor = f32::from(match *format {
        DataFormat::S5 => S5_FACTOR,
        DataFormat::S7 => S7_FACTOR,
    });
    use AnalogUIRange::*;
    let data = (data as i16) as f32;

    #[cfg_attr(rustfmt, rustfmt_skip)]
    match *range {
        mA0To20         => (data * 20.0 / factor),
        mA4To20         => (data * 16.0 / factor + 4.0),
        V0To10          |
        VMinus10To10    => (data * 10.0 / factor),
        V0To5           |
        VMinus5To5      => (data * 5.0 / factor),
        V1To5           => (data * 4.0 / factor + 1.0),
        V2To10          => (data * 8.0 / factor + 2.0),
        Disabled        => 0.0,
    }
}

fn parameters_from_raw_data(data: &[u16]) -> Result<Vec<ChannelParameters>> {
    if data.len() < 12 {
        return Err(Error::BufferLength);
    }

    let channel_parameters: Result<Vec<_>> = (0..4)
        .map(|i| {
            let mut p = ChannelParameters::default();
            let idx = i * 3;

            p.data_format = match FromPrimitive::from_u16(data[idx]) {
                Some(x) => x,
                _ => {
                    return Err(Error::ChannelParameter);
                }
            };

            p.output_range = match FromPrimitive::from_u16(data[idx + 1]) {
                Some(x) => x,
                _ => {
                    return Err(Error::ChannelParameter);
                }
            };
            p.substitute_value = u16_to_value(data[idx + 2], &p.output_range, &p.data_format);

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
    fn test_u16_to_value() {
        assert_eq!(
            u16_to_value(0x3600, &AnalogUIRange::mA0To20, &DataFormat::S7),
            10.0
        );
        assert_eq!(
            u16_to_value(0x2000, &AnalogUIRange::mA0To20, &DataFormat::S5),
            10.0
        );

        assert_eq!(
            u16_to_value(0x3600, &AnalogUIRange::mA4To20, &DataFormat::S7),
            12.0
        );
        assert_eq!(
            u16_to_value(0x2000, &AnalogUIRange::mA4To20, &DataFormat::S5),
            12.0
        );

        assert_eq!(
            u16_to_value(0x3600, &AnalogUIRange::V0To10, &DataFormat::S7),
            5.0
        );
        assert_eq!(
            u16_to_value(0x2000, &AnalogUIRange::V0To10, &DataFormat::S5),
            5.0
        );

        assert_eq!(
            u16_to_value(0x3600, &AnalogUIRange::VMinus10To10, &DataFormat::S7),
            5.0
        );
        assert_eq!(
            u16_to_value(0x2000, &AnalogUIRange::VMinus10To10, &DataFormat::S5),
            5.0
        );

        assert_eq!(
            u16_to_value(0x3600, &AnalogUIRange::V2To10, &DataFormat::S7),
            6.0
        );
        assert_eq!(
            u16_to_value(0x2000, &AnalogUIRange::V2To10, &DataFormat::S5),
            6.0
        );

        assert_eq!(
            u16_to_value(0x3600, &AnalogUIRange::V1To5, &DataFormat::S7),
            3.0
        );
        assert_eq!(
            u16_to_value(0x2000, &AnalogUIRange::V1To5, &DataFormat::S5),
            3.0
        );

        assert_eq!(
            u16_to_value(0x3600, &AnalogUIRange::V0To5, &DataFormat::S7),
            2.5
        );
        assert_eq!(
            u16_to_value(0x2000, &AnalogUIRange::V0To5, &DataFormat::S5),
            2.5
        );

        assert_eq!(
            u16_to_value(0x3600, &AnalogUIRange::VMinus5To5, &DataFormat::S7),
            2.5
        );
        assert_eq!(
            u16_to_value(0xCA00, &AnalogUIRange::VMinus5To5, &DataFormat::S7),
            -2.5
        );
        assert_eq!(
            u16_to_value(0x2000, &AnalogUIRange::VMinus5To5, &DataFormat::S5),
            2.5
        );
        assert_eq!(
            u16_to_value(0xE000, &AnalogUIRange::VMinus5To5, &DataFormat::S5),
            -2.5
        );
    }

    #[test]
    fn test_channel_parameters_from_raw_data() {
        let data = vec![
            1, 8, 0,        // CH 0
            1, 0, 0,        // CH 1
            0, 2, 0,        // CH 2
            1, 5, 0xCA00,   // CH 3
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
            parameters_from_raw_data(&data).unwrap()[2].output_range,
            AnalogUIRange::V0To10
        );

        assert_eq!(
            parameters_from_raw_data(&data).unwrap()[3].substitute_value,
            -2.5
        );
    }

    #[test]
    fn test_parameters_from_invalid_raw_data() {
        let mut data = vec![
            0, 0, 0,  // CH 0
            0, 0, 0,  // CH 1
            0, 0, 0,  // CH 2
            0, 0, 0,  // CH 3
        ];
        data[0] = 2; // should be max '1'
        assert!(parameters_from_raw_data(&data).is_err());

        data[0] = 0;
        data[1] = 9; // should be max '8'
        assert!(parameters_from_raw_data(&data).is_err());
    }

    #[test]
    fn test_parameters_from_invalid_data_buffer_size() {
        let data = [0; 0];
        assert!(parameters_from_raw_data(&data).is_err());
        let data = [0; 11];
        assert!(parameters_from_raw_data(&data).is_err());
        let data = [0; 12];
        assert!(parameters_from_raw_data(&data).is_ok());
    }

    #[test]
    fn create_module_from_parameter_data() {
        let data = vec![
            1, 0, 0,  // CH 0
            0, 8, 0,  // CH 1
            0, 0, 0,  // CH 2
            0, 0, 0,  // CH 3
        ];
        let module = Mod::from_parameter_data(&data).unwrap();
        assert_eq!(module.ch_params[0].data_format, DataFormat::S7);
        assert_eq!(module.ch_params[1].output_range, AnalogUIRange::Disabled);
    }
}
