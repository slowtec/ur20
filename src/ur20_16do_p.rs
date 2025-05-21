//! Digital output module UR20-16DO-P

use super::*;
use crate::ur20_fbc_mod_tcp::{FromModbusParameterData, ProcessModbusTcpData};
use crate::util::*;

#[derive(Debug)]
pub struct Mod;

impl FromModbusParameterData for Mod {
    fn from_modbus_parameter_data(data: &[u16]) -> Result<Mod> {
        if !data.is_empty() {
            return Err(Error::BufferLength);
        }
        Ok(Mod)
    }
}

impl Default for Mod {
    fn default() -> Self {
        Mod
    }
}

impl Module for Mod {
    fn module_type(&self) -> ModuleType {
        ModuleType::UR20_16DO_P
    }
}

impl ProcessModbusTcpData for Mod {
    fn process_input_byte_count(&self) -> usize {
        0
    }
    fn process_output_byte_count(&self) -> usize {
        2
    }
    fn process_output_data(&self, data: &[u16]) -> Result<Vec<ChannelValue>> {
        if data.len() != 1 {
            return Err(Error::BufferLength);
        }
        Ok((0..16)
            .map(|i| test_bit_16(data[0], i))
            .map(ChannelValue::Bit)
            .collect())
    }
    fn process_output_values(&self, values: &[ChannelValue]) -> Result<Vec<u16>> {
        if values.len() != 16 {
            return Err(Error::ChannelValue);
        }
        let mut res = 0;
        for (i, v) in values.iter().enumerate() {
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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::ChannelValue::*;

    #[test]
    fn test_process_output_values_with_invalid_channel_len() {
        let m = Mod;
        assert!(m.process_output_values(&[]).is_err());
        assert!(m.process_output_values(&vec![Bit(true); 15]).is_err());
        assert!(m.process_output_values(&vec![Bit(true); 16]).is_ok());
    }

    #[test]
    fn test_process_output_data() {
        let m = Mod;
        assert_eq!(
            m.process_output_data(&[0xFFFF]).unwrap(),
            vec![ChannelValue::Bit(true); 16]
        );
        let res = m.process_output_data(&[0b_0010_0001_0010_0101]).unwrap();
        assert_eq!(res[0], ChannelValue::Bit(true));
        assert_eq!(res[1], ChannelValue::Bit(false));
        assert_eq!(res[5], ChannelValue::Bit(true));
        assert_eq!(res[8], ChannelValue::Bit(true));
        assert_eq!(res[13], ChannelValue::Bit(true));
    }

    #[test]
    fn test_process_output_data_with_invalid_buffer_size() {
        let m = Mod;
        assert!(m.process_output_data(&[0; 2]).is_err());
    }

    #[test]
    fn test_process_output_values_with_invalid_channel_values() {
        let m = Mod;
        assert!(m.process_output_values(&vec![Decimal32(0.0); 16]).is_err());
    }

    #[test]
    fn test_process_output_values() {
        let m = Mod;
        let mut vals = vec![Bit(false); 16];
        vals[0] = Bit(true);
        vals[2] = Bit(true);
        vals[3] = Bit(true);
        vals[13] = Bit(true);

        assert_eq!(
            m.process_output_values(&vals).unwrap(),
            vec![0b0010_0000_0000_1101]
        );

        vals[7] = Bit(true);
        vals[8] = Bit(true);

        assert_eq!(
            m.process_output_values(&vals).unwrap(),
            vec![0b0010_0001_1000_1101]
        );
    }

    #[test]
    fn module_type() {
        let m = Mod;
        assert_eq!(m.module_type(), ModuleType::UR20_16DO_P);
    }

    #[test]
    fn create_module_from_modbus_parameter_data() {
        assert!(Mod::from_modbus_parameter_data(&[]).is_ok());
        assert!(Mod::from_modbus_parameter_data(&[0]).is_err());
    }
}
