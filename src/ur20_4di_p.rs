//! Digital input module UR20-4DI-P

use super::*;
use super::util::test_bit_16;
use num_traits::cast::FromPrimitive;

#[derive(Debug)]
pub struct Mod {
    pub ch_params: Vec<ChannelParameters>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelParameters {
    pub input_delay: InputDelay,
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
            input_delay: InputDelay::ms3,
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
    fn process_input_byte_count(&self) -> usize {
        1
    }
    fn process_output_byte_count(&self) -> usize {
        0
    }
    fn module_type(&self) -> ModuleType {
        ModuleType::UR20_4DI_P
    }
    fn process_input_data(&self, data: &[u16]) -> Result<Vec<ChannelValue>> {
        if data.len() != 1 {
            return Err(Error::BufferLength);
        }
        let bits = data[0];
        let res = (0..4)
            .map(|i| ChannelValue::Bit(test_bit_16(bits, i)))
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

fn parameters_from_raw_data(data: &[u16]) -> Result<Vec<ChannelParameters>> {
    if data.len() < 4 {
        return Err(Error::BufferLength);
    }

    let channel_parameters: Result<Vec<_>> = (0..4)
        .map(|i| {
            let mut p = ChannelParameters::default();
            p.input_delay = match FromPrimitive::from_u16(data[i]) {
                Some(x) => x,
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
        assert!(m.process_input_data(&vec![]).is_err());
        let data = vec![0b0100];
        assert_eq!(
            m.process_input_data(&data).unwrap(),
            vec![Bit(false), Bit(false), Bit(true), Bit(false)]
        );
    }

    #[test]
    fn test_process_output_values() {
        let m = Mod::default();
        assert!(m.process_output_values(&[ChannelValue::Bit(true)]).is_err());
        assert_eq!(m.process_output_values(&[]).unwrap(), &[]);
    }

    #[test]
    fn test_channel_parameters_from_raw_data() {
        let data = vec![
            2, // CH 0
            3, // CH 1
            4, // CH 2
            0, // CH 3
        ];

        assert_eq!(parameters_from_raw_data(&data).unwrap().len(), 4);

        assert_eq!(
            parameters_from_raw_data(&data).unwrap()[0],
            ChannelParameters::default()
        );

        assert_eq!(
            parameters_from_raw_data(&data).unwrap()[1].input_delay,
            InputDelay::ms10
        );

        assert_eq!(
            parameters_from_raw_data(&data).unwrap()[2].input_delay,
            InputDelay::ms20
        );

        assert_eq!(
            parameters_from_raw_data(&data).unwrap()[3].input_delay,
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
    fn create_module_from_parameter_data() {
        let data = vec![
            0, // CH 0
            3, // CH 1
            4, // CH 2
            5, // CH 3
        ];
        let module = Mod::from_parameter_data(&data).unwrap();
        assert_eq!(module.ch_params[0].input_delay, InputDelay::no);
        assert_eq!(module.ch_params[3].input_delay, InputDelay::ms40);
    }
}
