//! Digital input module UR20-4DI-P and UR20_8DI_P_2W

use std::marker::PhantomData;

use super::util::test_bit_16;
use super::*;
use crate::ur20_fbc_mod_tcp::{FromModbusParameterData, ProcessModbusTcpData};
use num_traits::cast::FromPrimitive;

trait DIVariant: Debug + Send {
    const MODULE_TYPE: ModuleType;
}

macro_rules! make_variants {
    ($(struct $name:ident,)*) => {
        $(
            #[allow(non_camel_case_types)]
            #[derive(Debug)]
            pub struct $name;
            impl DIVariant for $name {
                const MODULE_TYPE: ModuleType = ModuleType::$name;
            }
        )*
    };
}

make_variants! {
    struct UR20_4DI_P,
    struct UR20_8DI_P_2W,
}

#[derive(Debug)]
pub struct Mod<Variant> {
    pub ch_params: Vec<ChannelParameters>,
    _phantom: PhantomData<Variant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelParameters {
    pub input_delay: InputDelay,
}

impl<Variant: DIVariant> FromModbusParameterData for Mod<Variant> {
    fn from_modbus_parameter_data(data: &[u16]) -> Result<Self> {
        let ch_params = parameters_from_raw_data::<Variant>(data)?;
        Ok(Mod {
            ch_params,
            _phantom: PhantomData,
        })
    }
}

impl Default for ChannelParameters {
    fn default() -> Self {
        ChannelParameters {
            input_delay: InputDelay::ms3,
        }
    }
}

impl<Variant: DIVariant> Default for Mod<Variant> {
    fn default() -> Self {
        let ch_params = (0..Variant::MODULE_TYPE.channel_count())
            .map(|_| ChannelParameters::default())
            .collect();
        Mod {
            ch_params,
            _phantom: PhantomData,
        }
    }
}

impl<Variant: DIVariant> Module for Mod<Variant> {
    fn module_type(&self) -> ModuleType {
        Variant::MODULE_TYPE
    }
}

impl<Variant: DIVariant> ProcessModbusTcpData for Mod<Variant> {
    fn process_input_byte_count(&self) -> usize {
        Variant::MODULE_TYPE.channel_count().div_ceil(8)
    }
    fn process_output_byte_count(&self) -> usize {
        0
    }
    fn process_input_data(&self, data: &[u16]) -> Result<Vec<ChannelValue>> {
        if data.len() != 1 {
            return Err(Error::BufferLength);
        }
        let bits = data[0];
        let res = (0..Variant::MODULE_TYPE.channel_count())
            .map(|i| ChannelValue::Bit(test_bit_16(bits, i)))
            .collect();
        Ok(res)
    }
}

fn parameters_from_raw_data<Variant: DIVariant>(data: &[u16]) -> Result<Vec<ChannelParameters>> {
    if data.len() < Variant::MODULE_TYPE.channel_count() {
        return Err(Error::BufferLength);
    }

    let channel_parameters: Result<Vec<_>> = (0..Variant::MODULE_TYPE.channel_count())
        .map(|i| {
            let p = ChannelParameters {
                input_delay: match FromPrimitive::from_u16(data[i]) {
                    Some(x) => x,
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
    fn test_process_input_data() {
        let m = Mod::<UR20_4DI_P>::default();
        assert!(m.process_input_data(&[]).is_err());
        let data = vec![0b0100];
        assert_eq!(
            m.process_input_data(&data).unwrap(),
            vec![Bit(false), Bit(false), Bit(true), Bit(false)]
        );
    }

    #[test]
    fn test_process_input_data_8() {
        let m = Mod::<UR20_8DI_P_2W>::default();
        assert!(m.process_input_data(&[]).is_err());
        let data = vec![0b11010100];
        assert_eq!(
            m.process_input_data(&data).unwrap(),
            vec![
                Bit(false),
                Bit(false),
                Bit(true),
                Bit(false),
                Bit(true),
                Bit(false),
                Bit(true),
                Bit(true)
            ]
        );
    }

    #[test]
    fn test_process_output_data() {
        let m = Mod::<UR20_4DI_P>::default();
        assert!(m.process_output_data(&[0; 4]).is_err());
        assert_eq!(
            m.process_output_data(&[]).unwrap(),
            vec![ChannelValue::None; 4]
        );
    }

    #[test]
    fn test_process_output_values() {
        let m = Mod::<UR20_4DI_P>::default();
        assert!(m.process_output_values(&[ChannelValue::Bit(true)]).is_err());
        assert_eq!(m.process_output_values(&[]).unwrap(), &[]);
        assert_eq!(
            m.process_output_values(&vec![ChannelValue::None; 4])
                .unwrap(),
            &[]
        );
    }

    #[test]
    fn test_channel_parameters_from_raw_data() {
        let data = vec![
            2, // CH 0
            3, // CH 1
            4, // CH 2
            0, // CH 3
        ];

        assert_eq!(
            parameters_from_raw_data::<UR20_4DI_P>(&data).unwrap().len(),
            4
        );

        assert_eq!(
            parameters_from_raw_data::<UR20_4DI_P>(&data).unwrap()[0],
            ChannelParameters::default()
        );

        assert_eq!(
            parameters_from_raw_data::<UR20_4DI_P>(&data).unwrap()[1].input_delay,
            InputDelay::ms10
        );

        assert_eq!(
            parameters_from_raw_data::<UR20_4DI_P>(&data).unwrap()[2].input_delay,
            InputDelay::ms20
        );

        assert_eq!(
            parameters_from_raw_data::<UR20_4DI_P>(&data).unwrap()[3].input_delay,
            InputDelay::no
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
        data[0] = 6; // should be max '5'
        assert!(parameters_from_raw_data::<UR20_4DI_P>(&data).is_err());
    }

    #[test]
    fn test_parameters_from_invalid_data_buffer_size() {
        let data = [0; 0];
        assert!(parameters_from_raw_data::<UR20_4DI_P>(&data).is_err());
        let data = [0; 3];
        assert!(parameters_from_raw_data::<UR20_4DI_P>(&data).is_err());
        let data = [0; 4];
        assert!(parameters_from_raw_data::<UR20_4DI_P>(&data).is_ok());
    }

    #[test]
    fn create_module_from_modbus_parameter_data() {
        let data = vec![
            0, // CH 0
            3, // CH 1
            4, // CH 2
            5, // CH 3
        ];
        let module = Mod::<UR20_4DI_P>::from_modbus_parameter_data(&data).unwrap();
        assert_eq!(module.ch_params[0].input_delay, InputDelay::no);
        assert_eq!(module.ch_params[3].input_delay, InputDelay::ms40);
    }

    #[test]
    fn create_module_from_modbus_parameter_data_8() {
        let data = vec![0, 1, 2, 3, 4, 5, 2, 2];
        let module = Mod::<UR20_8DI_P_2W>::from_modbus_parameter_data(&data).unwrap();
        let input_delays = module
            .ch_params
            .into_iter()
            .map(|p| p.input_delay)
            .collect::<Vec<_>>();
        assert_eq!(
            input_delays,
            [
                InputDelay::no,
                InputDelay::us300,
                InputDelay::ms3,
                InputDelay::ms10,
                InputDelay::ms20,
                InputDelay::ms40,
                InputDelay::ms3,
                ChannelParameters::default().input_delay
            ]
        );
    }
}
