//! Digital output module UR20-4DO-P

use super::*;
use ur20_fbc_mod_tcp::{FromModbusParameterData, ProcessModbusTcpData};
use util::*;

#[derive(Debug)]
pub struct Mod {
    pub ch_params: Vec<ChannelParameters>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelParameters {
    pub substitute_value: bool,
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
            substitute_value: false,
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
        ModuleType::UR20_4DO_P
    }
}
impl ProcessModbusTcpData for Mod {
    fn process_input_byte_count(&self) -> usize {
        0
    }
    fn process_output_byte_count(&self) -> usize {
        1
    }
    fn process_output_data(&self, data: &[u16]) -> Result<Vec<ChannelValue>> {
        if data.len() != 1 {
            return Err(Error::BufferLength);
        }
        Ok((0..4)
            .map(|i| test_bit_16(data[0], i))
            .map(ChannelValue::Bit)
            .collect())
    }
    fn process_output_values(&self, values: &[ChannelValue]) -> Result<Vec<u16>> {
        if values.len() != 4 {
            return Err(Error::ChannelValue);
        }
        let mut res = 0;
        for (i, v) in values.into_iter().enumerate() {
            match *v {
                ChannelValue::Bit(state) => {
                    if state {
                        res = set_bit_16(res, i);
                    }
                }
                ChannelValue::Disabled => {
                    // do nothing
                }
                _ => {
                    return Err(Error::ChannelValue);
                }
            }
        }
        Ok(vec![res])
    }
}

fn parameters_from_raw_data(data: &[u16]) -> Result<Vec<ChannelParameters>> {
    if data.len() < 4 {
        return Err(Error::BufferLength);
    }

    let channel_parameters: Result<Vec<_>> = (0..4)
        .map(|i| {
            let mut p = ChannelParameters::default();
            p.substitute_value = match data[i] {
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
    fn test_process_output_values_with_invalid_channel_len() {
        let m = Mod::default();
        assert!(m.process_output_values(&[]).is_err());
        assert!(
            m.process_output_values(&[Bit(true), Bit(false), Bit(true)])
                .is_err()
        );
        assert!(
            m.process_output_values(&[Bit(true), Bit(false), Bit(true), Bit(true)])
                .is_ok()
        );
    }

    #[test]
    fn test_process_output_data() {
        let m = Mod::default();
        assert_eq!(
            m.process_output_data(&vec![0x0F]).unwrap(),
            &[
                ChannelValue::Bit(true),
                ChannelValue::Bit(true),
                ChannelValue::Bit(true),
                ChannelValue::Bit(true),
            ]
        );
        assert_eq!(
            m.process_output_data(&vec![0b000_0101]).unwrap(),
            &[
                ChannelValue::Bit(true),
                ChannelValue::Bit(false),
                ChannelValue::Bit(true),
                ChannelValue::Bit(false),
            ]
        );
    }

    #[test]
    fn test_process_output_data_with_invalid_buffer_size() {
        let m = Mod::default();
        assert!(m.process_output_data(&vec![0; 2]).is_err());
    }

    #[test]
    fn test_process_output_values_with_invalid_channel_values() {
        let m = Mod::default();
        assert!(
            m.process_output_values(&[Bit(false), Bit(true), Bit(false), Decimal32(0.0)])
                .is_err()
        );
    }

    #[test]
    fn test_process_output_values() {
        let m = Mod::default();
        assert_eq!(
            m.process_output_values(&[Bit(true), Bit(false), Bit(true), Bit(true)])
                .unwrap(),
            vec![0b0000_0000_0000_0000_1101]
        );

        assert_eq!(
            m.process_output_values(&[Bit(true), Bit(false), Bit(true), Bit(true)])
                .unwrap(),
            vec![0b0000_0000_0000_0000_1101]
        );
    }
    #[test]
    fn module_type() {
        let m = Mod::default();
        assert_eq!(m.module_type(), ModuleType::UR20_4DO_P);
    }

    #[test]
    fn test_channel_parameters_from_raw_data() {
        let data = vec![
            0, // CH 0
            1, // CH 1
            0, // CH 2
            1, // CH 3
        ];

        assert_eq!(parameters_from_raw_data(&data).unwrap().len(), 4);

        assert_eq!(
            parameters_from_raw_data(&data).unwrap()[0],
            ChannelParameters::default()
        );

        assert_eq!(
            parameters_from_raw_data(&data).unwrap()[1].substitute_value,
            true
        );

        assert_eq!(
            parameters_from_raw_data(&data).unwrap()[2].substitute_value,
            false
        );

        assert_eq!(
            parameters_from_raw_data(&data).unwrap()[3].substitute_value,
            true
        );
    }

    #[test]
    fn test_parameters_from_invalid_raw_data() {
        let mut data = vec![
            0, // CH 0
            0, // CH 1
            0, // CH 2
            0, // CH 3
        ];
        data[0] = 2; // should be max '1'
        assert!(parameters_from_raw_data(&data).is_err());
    }

    #[test]
    fn test_parameters_from_invalid_data_buffer_size() {
        let data = [0; 0];
        assert!(parameters_from_raw_data(&data).is_err());
        let data = [0; 3];
        assert!(parameters_from_raw_data(&data).is_err());
        let data = [0; 4];
        assert!(parameters_from_raw_data(&data).is_ok());
    }

    #[test]
    fn create_module_from_modbus_parameter_data() {
        let data = vec![
            1, // CH 0
            0, // CH 1
            1, // CH 2
            0, // CH 3
        ];
        let module = Mod::from_modbus_parameter_data(&data).unwrap();
        assert_eq!(module.ch_params[0].substitute_value, true);
        assert_eq!(module.ch_params[3].substitute_value, false);
    }
}
