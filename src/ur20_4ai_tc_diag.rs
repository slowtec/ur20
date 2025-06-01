//! Analog input module UR20-4AI-TC-DIAG
//!
//! This module returns 4 f32 input channels in V, if the measurement range is set up as a voltage range,
//! or in the configured temperature unit if a Thermocouple Type is given.
//!
//! It also provides 4 mock output channels which are always None. This mirrors what other modules do.

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

impl ChannelParameters {
    fn is_enabled(&self) -> bool {
        !matches!(self.measurement_range, MeasurementRange::Disabled)
    }
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

    /// ±15.625 mV
    uVPlusMinus15625 = 11,
    /// ±31.25 mV
    uVPlusMinus31250 = 12,
    /// ±62.5 mV
    uVPlusMinus62500 = 13,
    /// ±125 mV
    mVPlusMinus125 = 14,
    /// ±250 mV
    mVPlusMinus250 = 15,
    /// ±500 mV
    mVPlusMinus500 = 16,
    /// ±1 V
    VPlusMinus1 = 17,
    /// ±2 V
    VPlusMinus2 = 18,

    #[default]
    Disabled = 19,
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
            measurement_range: MeasurementRange::Disabled,
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
            .map(|(&val, cfg)| {
                if cfg.is_enabled() {
                    match u16_to_thermal_value(val, cfg.measurement_range) {
                        Some(v) => ChannelValue::Decimal32(v),
                        None => ChannelValue::None,
                    }
                } else {
                    ChannelValue::Disabled
                }
            })
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
        Disabled => None, // This is actually Disabled, not None, but handled in `process_input_data`.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_input_data_wrong_size() {
        let m = Mod::default();
        assert!(m.process_input_data(&[0; 0]).is_err());
        assert!(m.process_input_data(&[0; 1]).is_err());
        assert!(m.process_input_data(&[0; 2]).is_err());
        assert!(m.process_input_data(&[0; 3]).is_err());
        assert!(m.process_input_data(&[0; 4]).is_ok());
        assert!(m.process_input_data(&[0; 5]).is_err());
    }

    /// Helper function for approximate float comparison
    fn assert_approx_equal(a: &ChannelValue, b: &ChannelValue) {
        match (a, b) {
            (ChannelValue::Decimal32(x), ChannelValue::Decimal32(y)) => {
                let epsilon = 1e-4; // Small tolerance for floating point comparisons
                assert!(
                    (x - y).abs() <= epsilon,
                    "Expected {a:?} to be approximately equal to {b:?}"
                );
            }
            _ => assert_eq!(a, b),
        }
    }
    
    #[test]
    fn test_process_input_data() {
        let mut m = Mod::default();
        assert_eq!(m.ch_params[0].measurement_range, MeasurementRange::Disabled);
        assert_eq!(m.ch_params[1].measurement_range, MeasurementRange::Disabled);
        assert_eq!(m.ch_params[2].measurement_range, MeasurementRange::Disabled);
        assert_eq!(m.ch_params[3].measurement_range, MeasurementRange::Disabled);
        assert_eq!(
            m.process_input_data(&[0, 0, 0, 0]).unwrap(),
            vec![ChannelValue::Disabled; 4]
        );
    
        // Set up channels with different measurement ranges
        m.ch_params[0].measurement_range = MeasurementRange::TC_Type_K;  // Temperature
        m.ch_params[1].measurement_range = MeasurementRange::VPlusMinus2;  // Voltage ±2V
        m.ch_params[2].measurement_range = MeasurementRange::mVPlusMinus500;  // Voltage ±500mV
        m.ch_params[3].measurement_range = MeasurementRange::uVPlusMinus15625;  // Voltage ±15.625µV
    
        // Test temperature conversion (K-type thermocouple)
        let results = m.process_input_data(&[-2000_i16 as u16, 0, 0, 0]).unwrap();
        assert_approx_equal(&results[0], &ChannelValue::Decimal32(-200.0));
        assert_approx_equal(&results[1], &ChannelValue::Decimal32(0.0));
        assert_approx_equal(&results[2], &ChannelValue::Decimal32(0.0));
        assert_approx_equal(&results[3], &ChannelValue::Decimal32(0.0));
    
        // Test voltage conversions
        let results = m.process_input_data(&[0, 16384, 16384, 16384]).unwrap();
        assert_approx_equal(&results[0], &ChannelValue::Decimal32(0.0));
        assert_approx_equal(&results[1], &ChannelValue::Decimal32(1.0));
        assert_approx_equal(&results[2], &ChannelValue::Decimal32(0.25));
        assert_approx_equal(&results[3], &ChannelValue::Decimal32(0.0078125));
    
        // Test line break detection (32767 for thermocouples)
        let results = m.process_input_data(&[32767, 32767, 32767, 32767]).unwrap();
        assert_eq!(results[0], ChannelValue::None);
        assert_approx_equal(&results[1], &ChannelValue::Decimal32(2.0));
        assert_approx_equal(&results[2], &ChannelValue::Decimal32(0.5));
        assert_approx_equal(&results[3], &ChannelValue::Decimal32(0.015625));
    
        // Test single disabled channel
        m.ch_params[0].measurement_range = MeasurementRange::Disabled;
        let results = m.process_input_data(&[1000, 0, 0, 0]).unwrap();
        assert_eq!(results[0], ChannelValue::Disabled);
        assert_approx_equal(&results[1], &ChannelValue::Decimal32(0.0));
        assert_approx_equal(&results[2], &ChannelValue::Decimal32(0.0));
        assert_approx_equal(&results[3], &ChannelValue::Decimal32(0.0));
    }

    #[test]
    fn test_process_input_data_with_underloading() {
        let mut m = Mod::default();

        m.ch_params[0].measurement_range = MeasurementRange::TC_Type_B; // 50 - 1820°C
        m.ch_params[1].measurement_range = MeasurementRange::TC_Type_J; // -210 - 1200°C

        let inputs = m.process_input_data(&[460, -2140_i16 as u16, 0, 0]).unwrap();

        assert_approx_equal(&inputs[0], &ChannelValue::Decimal32(46.0));
        assert_approx_equal(&inputs[1], &ChannelValue::Decimal32(-214.0));
    }

    /// Tests decoding of the 4 mock output channels
    #[test]
    fn test_process_output_data() {
        let m = Mod::default();
        assert!(m.process_output_data(&[0; 4]).is_err());
        assert_eq!(
            m.process_output_data(&[]).unwrap(),
            vec![ChannelValue::None; 4]
        );
    }

    /// Tests encoding of the 4 mock output channels
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
        let mut data = vec![0; 1 + 4 * 7]; // 1 module param + 4 channels * 7 params each
        assert_eq!(
            parameters_from_raw_data(&data)
                .unwrap()
                .0
                .temperature_unit,
            TemperatureUnit::Celsius
        );
        data[0] = 1;
        assert_eq!(
            parameters_from_raw_data(&data)
                .unwrap()
                .0
                .temperature_unit,
            TemperatureUnit::Fahrenheit
        );
    }

    #[test]
    fn test_channel_parameters_from_raw_data() {
        #[rustfmt::skip]
        let data = vec![
            0,             // Module temperature unit (Celsius)
            // CH 0 (7 params)
            19, 0, 2, 0, 0, 32767, -32768_i16 as u16, 
            // CH 1 (7 params)
            1, 1, 1, 1, 1, 1000, -1000_i16 as u16, 
            // CH 2 (7 params)
            19, 0, 0, 0, 0, 0, 0,
            // CH 3 (7 params)
            11, 4, 2, 0, 0, 0, 0,
        ];

        let (_, ch_params) = parameters_from_raw_data(&data).unwrap();
        assert_eq!(ch_params.len(), 4);

        assert_eq!(
            ch_params[0],
            ChannelParameters::default()
        );

        assert!(ch_params[1].channel_diagnostics);
        assert!(ch_params[1].limit_value_monitoring);
        assert_eq!(ch_params[1].high_limit_value, 1000);
        assert_eq!(ch_params[1].low_limit_value, -1000);
        assert_eq!(
            ch_params[1].measurement_range,
            MeasurementRange::TC_Type_K
        );
        assert_eq!(
            ch_params[1].cold_junction_compensation,
            ColdJunctionCompensation::ExternalChannel0
        );
        assert_eq!(
            ch_params[1].conversion_time,
            ConversionTime::ms130
        );

        assert_eq!(
            ch_params[2].measurement_range,
            MeasurementRange::Disabled
        );

        assert_eq!(
            ch_params[3].measurement_range,
            MeasurementRange::uVPlusMinus15625
        );
        assert_eq!(
            ch_params[3].cold_junction_compensation,
            ColdJunctionCompensation::ExternalChannel3
        );
    }

    #[test]
    fn test_parameters_from_invalid_raw_data() {
        let mut data = vec![0; 1 + 4 * 7];

        // Invalid temperature unit
        data[0] = 3;
        assert!(parameters_from_raw_data(&data).is_err());
        // Reset to valid
        data[0] = 0;
        
        // Invalid measurement range
        data[1] = 20;
        assert!(parameters_from_raw_data(&data).is_err());
        data[1] = 0;
        
        // Invalid cold junction compensation
        data[2] = 5;
        assert!(parameters_from_raw_data(&data).is_err());
        data[2] = 0;
        
        // Invalid conversion time
        data[3] = 6;
        assert!(parameters_from_raw_data(&data).is_err());
        data[3] = 0;
        
        // Invalid channel diagnostics (must be 0 or 1)
        data[4] = 2;
        assert!(parameters_from_raw_data(&data).is_err());
        data[4] = 0;
        
        // Invalid limit value monitoring (must be 0 or 1)
        data[5] = 2;
        assert!(parameters_from_raw_data(&data).is_err());
    }

    #[test]
    fn test_parameters_from_invalid_data_buffer_size() {
        let data = [0; 0];
        assert!(parameters_from_raw_data(&data).is_err());
        let data = [0; 28]; // 1 + 4*7 - 1
        assert!(parameters_from_raw_data(&data).is_err());
        let data = [0; 29]; // 1 + 4*7
        assert!(parameters_from_raw_data(&data).is_ok());
    }

    #[test]
    fn create_module_from_modbus_parameter_data() {
        #[rustfmt::skip]
        let data = vec![
            1,             // Module (Fahrenheit)
            // CH 0 (7 params)
            0, 0, 0, 0, 0, 32767, -32768_i16 as u16,
            // CH 1 (7 params)
            19, 0, 0, 0, 0, 0, 0,  // Disabled
            // CH 2 (7 params)
            1, 0, 0, 1, 0, 0, 0,   // TC_Type_K with diagnostics
            // CH 3 (7 params)
            0, 0, 0, 0, 0, 0, 0,
        ];
        let module = Mod::from_modbus_parameter_data(&data).unwrap();
        assert_eq!(module.mod_params.temperature_unit, TemperatureUnit::Fahrenheit);
        assert_eq!(
            module.ch_params[0].measurement_range,
            MeasurementRange::TC_Type_J
        );
        assert_eq!(
            module.ch_params[1].measurement_range,
            MeasurementRange::Disabled
        );
        assert!(module.ch_params[2].channel_diagnostics);
    }
}
