//! Safe power-feed module UR20-PF-O-1DI-SIL

use super::*;
use crate::{
    ur20_fbc_mod_tcp::{FromModbusParameterData, ProcessModbusTcpData},
    util::test_bit,
};

#[derive(Debug, Clone, Default)]
pub struct Mod;

// Note: this is a subset of the 2DI_SIL and 2DI-DELAY-SIL config, can be extended.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProcessInput {
    /// Bytes 0 Bit 0: Safety input 0, `false`: inactive, `true`: active
    pub safety_input: bool,
    /// Bytes 0 Bit 2: Autostart, `false`: inactive, `true`: active
    pub autostart: bool,
    /// Bytes 0 Bit 3: Manual start, `false`: inactive, `true`: active
    pub manual_start: bool,
    /// Bytes 0 Bit 4: Safety input 0, channel 1, `false`: inactive, `true`: active
    pub safety_input_channel_1: bool,
    /// Bytes 0 Bit 5: Safety input 0, channel 2, `false`: inactive, `true`: active
    pub safety_input_channel_2: bool,
    /// Bytes 1 Bit 0: 24 V Safe output, `false`: inactive, `true`: active
    pub volt_24_safe_output: bool,
    /// Bytes 1 Bit 2: 24 V DC, `false`: no feed-in, `true`: power feed-in pending
    pub volt_24_dc: bool,
    // Byte 2 & 3 unused.
}

impl From<ProcessInput> for ChannelValue {
    fn from(o: ProcessInput) -> Self {
        ChannelValue::SilPFIn(o)
    }
}

impl Module for Mod {
    fn module_type(&self) -> ModuleType {
        ModuleType::UR20_PF_O_1DI_SIL
    }
}

impl FromModbusParameterData for Mod {
    fn from_modbus_parameter_data(data: &[u16]) -> Result<Mod> {
        if !data.is_empty() {
            return Err(Error::BufferLength);
        }
        Ok(Mod)
    }
}

impl ProcessModbusTcpData for Mod {
    fn process_input_byte_count(&self) -> usize {
        4
    }
    fn process_output_byte_count(&self) -> usize {
        0
    }
    fn process_input_data(&self, data: &[u16]) -> Result<Vec<ChannelValue>> {
        if data.len() != 2 {
            return Err(Error::BufferLength);
        }
        let [byte0, byte1] = data[0].to_le_bytes();
        let [_byte2, _byte3] = data[1].to_le_bytes(); // reserved
        Ok(vec![
            ProcessInput {
                safety_input: test_bit(byte0, 0),
                autostart: test_bit(byte0, 2),
                manual_start: test_bit(byte0, 3),
                safety_input_channel_1: test_bit(byte0, 4),
                safety_input_channel_2: test_bit(byte0, 5),
                volt_24_safe_output: test_bit(byte1, 0),
                volt_24_dc: test_bit(byte1, 2),
            }
            .into(),
        ])
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn process_input_byte_count() {
        let m = Mod;
        assert_eq!(m.process_input_byte_count(), 4);
    }

    #[test]
    fn process_output_byte_count() {
        let m = Mod;
        assert_eq!(m.process_output_byte_count(), 0);
    }

    #[test]
    fn test_process_input_data_with_invalid_buffer_size() {
        let m = Mod;
        assert!(m.process_input_data(&[0; 0]).is_err());
        assert!(m.process_input_data(&[0; 1]).is_err());
        assert!(m.process_input_data(&[0; 2]).is_ok());
        assert!(m.process_input_data(&[0; 3]).is_err());
        assert!(m.process_input_data(&[0; 4]).is_err());
    }

    #[test]
    fn test_process_input_data() {
        let m = Mod;
        let data = vec![0b0000_0101_0011_1101, 0];
        let res = m.process_input_data(&data).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(
            res[0],
            ChannelValue::SilPFIn(ProcessInput {
                safety_input: true,
                autostart: true,
                manual_start: true,
                safety_input_channel_1: true,
                safety_input_channel_2: true,
                volt_24_safe_output: true,
                volt_24_dc: true,
            })
        );

        let data = vec![0b0000_0001_0001_1000, 0];
        let res = m.process_input_data(&data).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(
            res[0],
            ChannelValue::SilPFIn(ProcessInput {
                safety_input: false,
                autostart: false,
                manual_start: true,
                safety_input_channel_1: true,
                safety_input_channel_2: false,
                volt_24_safe_output: true,
                volt_24_dc: false,
            })
        );

        let data = vec![0, 0];
        let res = m.process_input_data(&data).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(
            res[0],
            ChannelValue::SilPFIn(ProcessInput {
                safety_input: false,
                autostart: false,
                manual_start: false,
                safety_input_channel_1: false,
                safety_input_channel_2: false,
                volt_24_safe_output: false,
                volt_24_dc: false,
            })
        );
    }

    #[test]
    fn test_process_output_data_with_invalid_buffer_size() {
        let m = Mod;
        assert!(m.process_output_data(&[]).is_ok());
        assert!(m.process_output_data(&[0; 1]).is_err());
        assert!(m.process_output_data(&[0; 2]).is_err());
        assert!(m.process_output_data(&[0; 3]).is_err());
    }
}
