//! Digital output module UR20-4DO-P, UR20-4DO-P-2A, UR20-8DO-P

use std::marker::PhantomData;

use super::*;
use crate::ur20_fbc_mod_tcp::{FromModbusParameterData, ProcessModbusTcpData};
use crate::util::*;

trait DOVariant: Debug + Send {
    const MODULE_TYPE: ModuleType;
}

macro_rules! make_variants {
    ($(struct $name:ident),*) => {
        $(
            #[allow(non_camel_case_types)]
            #[derive(Debug)]
            pub struct $name;
            impl DOVariant for $name {
                const MODULE_TYPE: ModuleType = ModuleType::$name;
            }
        )*
    };
}

make_variants! {
    struct UR20_4DO_P,
    struct UR20_4DO_P_2A,
    struct UR20_8DO_P
}

#[derive(Debug)]
pub struct Mod<Variant> {
    pub ch_params: Vec<ChannelParameters>,
    _phantom: PhantomData<Variant>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChannelParameters {
    pub substitute_value: bool,
}

impl<Variant: DOVariant> FromModbusParameterData for Mod<Variant> {
    fn from_modbus_parameter_data(data: &[u16]) -> Result<Self> {
        let ch_params = parameters_from_raw_data::<Variant>(data)?;
        Ok(Self {
            ch_params,
            _phantom: PhantomData,
        })
    }
}

impl<Variant: DOVariant> Default for Mod<Variant> {
    fn default() -> Self {
        Mod {
            ch_params: vec![ChannelParameters::default(); Variant::MODULE_TYPE.channel_count()],
            _phantom: PhantomData,
        }
    }
}

impl<Variant: DOVariant> Module for Mod<Variant> {
    fn module_type(&self) -> ModuleType {
        Variant::MODULE_TYPE
    }
}
impl<Variant: DOVariant> ProcessModbusTcpData for Mod<Variant> {
    fn process_input_byte_count(&self) -> usize {
        0
    }
    fn process_output_byte_count(&self) -> usize {
        match Variant::MODULE_TYPE.channel_count() {
            4 => 1,
            8 => 1,
            16 => 2,
            _ => {
                unreachable!("Generic DO module should only be implemented for 4, 8, or 16 outputs")
            }
        }
    }
    fn process_output_data(&self, data: &[u16]) -> Result<Vec<ChannelValue>> {
        if data.len() != 1 {
            return Err(Error::BufferLength);
        }
        Ok((0..Variant::MODULE_TYPE.channel_count())
            .map(|i| test_bit_16(data[0], i))
            .map(ChannelValue::Bit)
            .collect())
    }
    fn process_output_values(&self, values: &[ChannelValue]) -> Result<Vec<u16>> {
        if values.len() != Variant::MODULE_TYPE.channel_count() {
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

fn parameters_from_raw_data<Variant: DOVariant>(data: &[u16]) -> Result<Vec<ChannelParameters>> {
    if data.len() < Variant::MODULE_TYPE.channel_count() {
        return Err(Error::BufferLength);
    }

    let channel_parameters: Result<Vec<_>> = (0..Variant::MODULE_TYPE.channel_count())
        .map(|i| {
            let p = ChannelParameters {
                substitute_value: match data[i] {
                    0 => false,
                    1 => true,
                    _ => {
                        return Err(Error::ChannelParameter);
                    }
                },
            };
            Ok(p)
        })
        .collect();
    channel_parameters
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::ChannelValue::*;

    #[test]
    fn test_process_output_values_with_invalid_channel_len() {
        let m = Mod::<UR20_4DO_P>::default();
        assert!(m.process_output_values(&[]).is_err());
        assert!(m
            .process_output_values(&[Bit(true), Bit(false), Bit(true)])
            .is_err());
        assert!(m
            .process_output_values(&[Bit(true), Bit(false), Bit(true), Bit(true)])
            .is_ok());
    }

    #[test]
    fn test_process_output_data() {
        let m = Mod::<UR20_4DO_P>::default();
        assert_eq!(
            m.process_output_data(&[0x0F]).unwrap(),
            &[
                ChannelValue::Bit(true),
                ChannelValue::Bit(true),
                ChannelValue::Bit(true),
                ChannelValue::Bit(true),
            ]
        );
        assert_eq!(
            m.process_output_data(&[0b000_0101]).unwrap(),
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
        let m = Mod::<UR20_4DO_P>::default();
        assert!(m.process_output_data(&[0; 2]).is_err());
    }

    #[test]
    fn test_process_output_values_with_invalid_channel_values() {
        let m = Mod::<UR20_4DO_P>::default();
        assert!(m
            .process_output_values(&[Bit(false), Bit(true), Bit(false), Decimal32(0.0)])
            .is_err());
    }

    #[test]
    fn test_process_output_values() {
        let m = Mod::<UR20_4DO_P>::default();
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
        let m = Mod::<UR20_4DO_P>::default();
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

        assert_eq!(
            parameters_from_raw_data::<UR20_4DO_P>(&data).unwrap().len(),
            4
        );

        assert_eq!(
            parameters_from_raw_data::<UR20_4DO_P>(&data).unwrap()[0],
            ChannelParameters::default()
        );

        assert!(parameters_from_raw_data::<UR20_4DO_P>(&data).unwrap()[1].substitute_value);

        assert!(!parameters_from_raw_data::<UR20_4DO_P>(&data).unwrap()[2].substitute_value);

        assert!(parameters_from_raw_data::<UR20_4DO_P>(&data).unwrap()[3].substitute_value);
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
        assert!(parameters_from_raw_data::<UR20_4DO_P>(&data).is_err());
    }

    #[test]
    fn test_parameters_from_invalid_data_buffer_size() {
        let data = [0; 0];
        assert!(parameters_from_raw_data::<UR20_4DO_P>(&data).is_err());
        let data = [0; 3];
        assert!(parameters_from_raw_data::<UR20_4DO_P>(&data).is_err());
        let data = [0; 4];
        assert!(parameters_from_raw_data::<UR20_4DO_P>(&data).is_ok());
    }

    #[test]
    fn create_module_from_modbus_parameter_data() {
        let data = vec![
            1, // CH 0
            0, // CH 1
            1, // CH 2
            0, // CH 3
        ];
        let module = Mod::<UR20_4DO_P>::from_modbus_parameter_data(&data).unwrap();
        assert!(module.ch_params[0].substitute_value);
        assert!(!module.ch_params[3].substitute_value);
    }
}
