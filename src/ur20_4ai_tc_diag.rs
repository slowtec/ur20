//! Analog input module UR20-4AI-TC-DIAG
//!
//! This module returns 4 f32 input channels in V, if the measurement range is set up as a voltage range,
//! or in the configured temperature unit if a Thermocouple Type is given.

use super::*;
use crate::ur20_fbc_mod_tcp::{FromModbusParameterData, ProcessModbusTcpData};
use num_traits::cast::FromPrimitive;

#[derive(Debug, Default)]
pub struct Mod {
    pub mod_params: ModuleParameters,
    pub ch_params: [ChannelParameters; 4],
}

impl Module for Mod {
    fn module_type(&self) -> ModuleType {
        ModuleType::UR20_4AI_TC_DIAG
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleParameters {
    pub temperature_unit: TemperatureUnit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelParameters {
    pub measurement_range: MeasurementRange,
    pub cold_junction_compensation: ColdJunctionCompensation,
    pub conversion_time: ConversionTime,
    pub channel_diagnostics: bool,
    pub limit_value_monitoring: bool,
    pub high_limit_value: i16,
    pub low_limit_value: i16,
}

/// Thermocouple measurement range
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, FromPrimitive, ToPrimitive)]
pub enum MeasurementRange {
    TC_Type_J = 0,
    TC_Type_K = 1,
    TC_Type_N = 2,
    TC_Type_R = 3,
    TC_Type_S = 4,
    TC_Type_T = 5,
    TC_Type_B = 6,
    TC_Type_C = 7,
    TC_Type_E = 8,
    TC_Type_L = 9,
    TC_Type_U = 10,

    uVPlusMinus15625 = 11,
    uVPlusMinus31250 = 12,
    uVPlusMinus62500 = 13,
    mVPlusMinus125 = 14,
    mVPlusMinus250 = 15,
    mVPlusMinus500 = 16,
    VPlusMinus1 = 17,
    VPlusMinus2 = 18,

    #[default]
    disabled = 19,
}

/// Cold Junction Compensation configurations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, FromPrimitive, ToPrimitive)]
pub enum ColdJunctionCompensation {
    #[default]
    Internal = 0,
    ExternalChannel0 = 1,
    ExternalChannel1 = 2,
    ExternalChannel2 = 3,
    ExternalChannel3 = 4,
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
            measurement_range: MeasurementRange::disabled,
            cold_junction_compensation: ColdJunctionCompensation::Internal,
            conversion_time: ConversionTime::ms80,
            channel_diagnostics: false,
            limit_value_monitoring: false,
            high_limit_value: 32767,
            low_limit_value: -32768,
        }
    }
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

impl ProcessModbusTcpData for Mod {
    fn process_input_byte_count(&self) -> usize {
        8 // 4 words
    }
    fn process_output_byte_count(&self) -> usize {
        0
    }
    fn process_input_data(&self, data: &[u16]) -> Result<Vec<ChannelValue>> {
        if data.len() != 4 {
            return Err(Error::BufferLength);
        }

        let res = data
            .iter()
            .zip(self.ch_params.iter())
            .map(
                |(&val, cfg)| match u16_to_thermal_value(val, cfg.measurement_range) {
                    Some(v) => ChannelValue::Decimal32(v),
                    None => ChannelValue::Disabled,
                },
            )
            .collect();
        Ok(res)
    }
}

fn parameters_from_raw_data(data: &[u16]) -> Result<(ModuleParameters, [ChannelParameters; 4])> {
    if data.len() < 1 + 4 * 7 {
        return Err(Error::BufferLength);
    }

    let temperature_unit = FromPrimitive::from_u16(data[0]).ok_or(Error::ChannelParameter)?;

    let module_parameters = ModuleParameters { temperature_unit };

    let channel_parameters = data[1..]
        .chunks_exact(7)
        .map(|data| {
            Ok(ChannelParameters {
                measurement_range: FromPrimitive::from_u16(data[0])
                    .ok_or(Error::ChannelParameter)?,
                cold_junction_compensation: FromPrimitive::from_u16(data[1])
                    .ok_or(Error::ChannelParameter)?,
                conversion_time: FromPrimitive::from_u16(data[2]).ok_or(Error::ChannelParameter)?,
                channel_diagnostics: match data[3] {
                    0 => false,
                    1 => true,
                    _ => {
                        return Err(Error::ChannelParameter);
                    }
                },
                limit_value_monitoring: match data[4] {
                    0 => false,
                    1 => true,
                    _ => {
                        return Err(Error::ChannelParameter);
                    }
                },
                high_limit_value: data[5] as i16,
                low_limit_value: data[6] as i16,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let channel_parameters = channel_parameters
        .try_into()
        .expect("Size was asserted above");

    Ok((module_parameters, channel_parameters))
}

/// Converts the input register to a voltage or temperature.
fn u16_to_thermal_value(val: u16, measurement_range: MeasurementRange) -> Option<f32> {
    use MeasurementRange::*;

    let fval = f32::from(val as i16);
    match measurement_range {
        // The module already does the proper conversion to the configured temperature range.
        // Under and Overloading are not handled here, as we rather give estimate information.
        TC_Type_J | TC_Type_K | TC_Type_N | TC_Type_R | TC_Type_S | TC_Type_T | TC_Type_B
        | TC_Type_C | TC_Type_E | TC_Type_L | TC_Type_U => {
            if val == 32767 {
                // Line break or cold compensation error
                None
            } else {
                Some(fval * 0.1)
            }
        }
        // For voltages, they are linear and no bounds exist. Converted to Volts.
        uVPlusMinus15625 => Some(0.015625 * fval / 32767.0),
        uVPlusMinus31250 => Some(0.031250 * fval / 32767.0),
        uVPlusMinus62500 => Some(0.062500 * fval / 32767.0),
        mVPlusMinus125 => Some(0.125 * fval / 32767.0),
        mVPlusMinus250 => Some(0.250 * fval / 32767.0),
        mVPlusMinus500 => Some(0.500 * fval / 32767.0),
        VPlusMinus1 => Some(1.0 * fval / 32767.0),
        VPlusMinus2 => Some(2.0 * fval / 32767.0),
        disabled => None,
    }
}

// #[cfg(test)]
// mod tests {

//     use super::*;
//     use crate::ChannelValue::*;

//     #[test]
//     fn test_process_input_data_with_empty_buffer() {
//         let m = Mod::default();
//         assert!(m.process_input_data(&[]).is_err());
//     }

//     #[test]
//     fn test_process_input_data_with_missing_channel_parameters() {
//         let m = Mod {
//             ch_params: vec![],
//             ..Default::default()
//         };
//         assert!(m.process_input_data(&[0; 4]).is_err());
//     }

//     #[test]
//     fn test_process_input_data() {
//         let mut m = Mod::default();
//         assert_eq!(m.ch_params[0].measurement_range, AnalogUIRange::Disabled);
//         assert_eq!(m.ch_params[1].measurement_range, AnalogUIRange::Disabled);
//         assert_eq!(m.ch_params[2].measurement_range, AnalogUIRange::Disabled);
//         assert_eq!(m.ch_params[3].measurement_range, AnalogUIRange::Disabled);
//         assert_eq!(
//             m.process_input_data(&[5, 0, 7, 8]).unwrap(),
//             vec![Disabled; 4]
//         );

//         m.ch_params[0].measurement_range = AnalogUIRange::mA0To20;
//         m.ch_params[1].measurement_range = AnalogUIRange::VMinus5To5;
//         m.ch_params[2].measurement_range = AnalogUIRange::V2To10;
//         m.ch_params[3].measurement_range = AnalogUIRange::V0To5;

//         m.ch_params[2].data_format = DataFormat::S5;

//         assert_eq!(
//             m.process_input_data(&[0x6C00, 0x3600, 0x4000, 0x6C00])
//                 .unwrap(),
//             vec![
//                 Decimal32(20.0),
//                 Decimal32(2.5),
//                 Decimal32(10.0),
//                 Decimal32(5.0),
//             ]
//         );
//     }

//     #[test]
//     fn test_process_input_data_with_underloading() {
//         let mut m = Mod::default();

//         m.ch_params[0].measurement_range = AnalogUIRange::mA4To20;
//         m.ch_params[0].data_format = DataFormat::S7;

//         m.ch_params[1].measurement_range = AnalogUIRange::mA4To20;
//         m.ch_params[1].data_format = DataFormat::S5;

//         let input = m.process_input_data(&[0xED00, 0x0F333, 0, 0]).unwrap();

//         if let ChannelValue::Decimal32(v) = input[0] {
//             assert!((v - 1.19).abs() < 0.01);
//         } else {
//             panic!();
//         }
//         if let ChannelValue::Decimal32(v) = input[1] {
//             assert!((v - 0.8).abs() < 0.01);
//         } else {
//             panic!();
//         }
//     }

//     #[test]
//     fn test_process_output_data() {
//         let m = Mod::default();
//         assert!(m.process_output_data(&[0; 4]).is_err());
//         assert_eq!(
//             m.process_output_data(&[]).unwrap(),
//             vec![ChannelValue::None; 4]
//         );
//     }

//     #[test]
//     fn test_process_output_values() {
//         let m = Mod::default();
//         assert!(
//             m.process_output_values(&[ChannelValue::Decimal32(0.0)])
//                 .is_err()
//         );
//         assert_eq!(m.process_output_values(&[]).unwrap(), &[]);
//         assert_eq!(
//             m.process_output_values(&vec![ChannelValue::None; 4])
//                 .unwrap(),
//             &[]
//         );
//     }

//     #[test]
//     fn test_module_parameters_from_raw_data() {
//         let mut data = vec![0; 21];
//         assert_eq!(
//             parameters_from_raw_data(&data)
//                 .unwrap()
//                 .0
//                 .frequency_suppression,
//             FrequencySuppression::Disabled
//         );
//         data[0] = 3;
//         assert_eq!(
//             parameters_from_raw_data(&data)
//                 .unwrap()
//                 .0
//                 .frequency_suppression,
//             FrequencySuppression::Average16
//         );
//     }

//     #[test]
//     fn test_channel_parameters_from_raw_data() {
//         #[rustfmt::skip]
//         let data = vec![
//             0,             // Module
//             0, 0, 0, 1, 8, // CH 0
//             1, 0, 0, 0, 5, // CH 1
//             0, 1, 0, 0, 0, // CH 2
//             0, 0, 1, 0, 0, // CH 3
//         ];

//         assert_eq!(parameters_from_raw_data(&data).unwrap().1.len(), 4);

//         assert_eq!(
//             parameters_from_raw_data(&data).unwrap().1[0],
//             ChannelParameters::default()
//         );

//         assert!(parameters_from_raw_data(&data).unwrap().1[1].channel_diagnostics);

//         assert!(!parameters_from_raw_data(&data).unwrap().1[1].diag_short_circuit);

//         assert!(!parameters_from_raw_data(&data).unwrap().1[1].diag_line_break);

//         assert_eq!(
//             parameters_from_raw_data(&data).unwrap().1[1].data_format,
//             DataFormat::S5
//         );

//         assert_eq!(
//             parameters_from_raw_data(&data).unwrap().1[1].measurement_range,
//             AnalogUIRange::VMinus5To5
//         );

//         assert!(parameters_from_raw_data(&data).unwrap().1[2].diag_short_circuit);
//         assert!(parameters_from_raw_data(&data).unwrap().1[3].diag_line_break);
//         assert_eq!(
//             parameters_from_raw_data(&data).unwrap().1[3].measurement_range,
//             AnalogUIRange::mA0To20
//         );
//     }

//     #[test]
//     fn test_parameters_from_invalid_raw_data() {
//         let mut data = vec![0; 21];

//         data[0] = 4; // should be max '3'
//         assert!(parameters_from_raw_data(&data).is_err());

//         data[0] = 0;
//         data[1] = 2; // should be '0' or '1'
//         assert!(parameters_from_raw_data(&data).is_err());

//         data[1] = 0;
//         data[2] = 2; // should be '0' or '1'
//         assert!(parameters_from_raw_data(&data).is_err());

//         data[2] = 0;
//         data[3] = 2; // should be '0' or '1'
//         assert!(parameters_from_raw_data(&data).is_err());

//         data[3] = 0;
//         data[4] = 2; // should be '0' or '1'
//         assert!(parameters_from_raw_data(&data).is_err());

//         data[4] = 0;
//         data[5] = 9; // should be max '8'
//         assert!(parameters_from_raw_data(&data).is_err());
//     }

//     #[test]
//     fn test_parameters_from_invalid_data_buffer_size() {
//         let data = [0; 0];
//         assert!(parameters_from_raw_data(&data).is_err());
//         let data = [0; 20];
//         assert!(parameters_from_raw_data(&data).is_err());
//         let data = [0; 21];
//         assert!(parameters_from_raw_data(&data).is_ok());
//     }

//     #[test]
//     fn create_module_from_modbus_parameter_data() {
//         #[rustfmt::skip]
//         let data = vec![
//             0,             // Module
//             0, 0, 0, 0, 1, // CH 0
//             0, 0, 0, 1, 8, // CH 1
//             1, 0, 0, 0, 0, // CH 2
//             0, 0, 0, 0, 0, // CH 3
//         ];
//         let module = Mod::from_modbus_parameter_data(&data).unwrap();
//         assert_eq!(
//             module.ch_params[0].measurement_range,
//             AnalogUIRange::mA4To20
//         );
//         assert_eq!(
//             module.ch_params[1].measurement_range,
//             AnalogUIRange::Disabled
//         );
//         assert!(module.ch_params[2].channel_diagnostics);
//     }
// }
