//! Analog input module UR20-4AI-RTD-DIAG

use super::*;

#[derive(Debug)]
pub struct Mod {
    pub mod_params: ModuleParameters,
    pub ch_params: Vec<ChannelParameters>,
}

#[derive(Debug, Clone)]
pub struct ModuleParameters {
    pub temperature_unit: TemperatureUnit,
}

#[derive(Debug, Clone)]
pub struct ChannelParameters {
    pub measurement_range: RtdRange,
    pub connection_type: ConnectionType,
    pub conversion_time: ConversionTime,
    pub channel_diagnostics: bool,
    pub limit_value_monitoring: bool,
    //-32768 ... 32767
    pub high_limit_value: u16,
    //-32768 ... 32767
    pub low_limit_value: u16,
}

impl Default for ModuleParameters {
    fn default() -> Self {
        ModuleParameters { temperature_unit: TemperatureUnit::Celsius }
    }
}

impl Default for ChannelParameters {
    fn default() -> Self {
        ChannelParameters {
            measurement_range: RtdRange::Disabled,
            connection_type: ConnectionType::TwoWire,
            conversion_time: ConversionTime::ms80,
            channel_diagnostics: false,
            limit_value_monitoring: false,
            high_limit_value: 0,
            low_limit_value: 0,
        }
    }
}

impl Default for Mod {
    fn default() -> Self {

        let ch_params = (0..4).map(|_| ChannelParameters::default()).collect();

        let mod_params = ModuleParameters::default();

        Mod {
            mod_params,
            ch_params,
        }
    }
}

impl Module for Mod {
    fn process_input_byte_count(&self) -> usize {
        8
    }
    fn process_output_byte_count(&self) -> usize {
        0
    }
    fn module_type(&self) -> ModuleType {
        ModuleType::UR20_4AI_RTD_DIAG
    }
    fn process_input_data(&self, data: &[u16]) -> Result<Vec<ChannelValue>> {
        if data.len() != 4 {
            return Err(Error::BufferLength);
        }

        if self.ch_params.len() != 4 {
            return Err(Error::ChannelParameter);
        }
        let res = (0..4)
            .map(|i| (data[i], &self.ch_params[i].measurement_range))
            .map(|(val, range)| {

                use RtdRange::*;

                match *range {
                    PT100  |
                    PT200  |
                    PT500  |
                    PT1000 |
                    NI100  |
                    NI120  |
                    NI200  |
                    NI500  |
                    NI1000 |
                    Cu10   => {
                        ChannelValue::Decimal32((val as i16) as f32 / 10.0)
                    }
                    R40   |
                    R80   |
                    R150  |
                    R300  |
                    R500  |
                    R1000 |
                    R2000 |
                    R4000 => {
                        let n = match *range {
                            R40   => 40.0,
                            R80   => 80.0,
                            R150  => 150.0,
                            R300  => 300.0,
                            R500  => 500.0,
                            R1000 => 1000.0,
                            R2000 => 2000.0,
                            R4000 => 4000.0,
                            _ => {
                                unreachable!();
                            }
                        };
                        let d = n * (val as u32) as f32 / 0x6C00 as f32;
                        ChannelValue::Decimal32(d)
                    }
                    Disabled => ChannelValue::Disabled,
                }
            })
            .collect();
        Ok(res)
    }
    fn process_output_values(&self, values: &[ChannelValue]) -> Result<Vec<u16>> {
        if values.len() != 0 {
            return Err(Error::ChannelValue);
        }
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use ChannelValue::*;

    #[test]
    fn test_process_input_data_with_empty_buffer() {
        let m = Mod::default();
        assert!(m.process_input_data(&vec![]).is_err());
    }

    #[test]
    fn test_process_input_data_with_missing_channel_parameters() {
        let mut m = Mod::default();
        m.ch_params = vec![];
        assert!(m.process_input_data(&vec![0, 0, 0, 0]).is_err());
    }

    #[test]
    fn test_process_input_data_with_disabled_channels() {
        let m = Mod::default();
        assert_eq!(
            m.process_input_data(&vec![5, 0, 7, 8]).unwrap(),
            vec![Disabled, Disabled, Disabled, Disabled]
        );
    }

    #[test]
    fn test_process_input_data() {
        let mut m = Mod::default();

        m.ch_params[0].measurement_range = RtdRange::R40;
        m.ch_params[1].measurement_range = RtdRange::R40;
        m.ch_params[2].measurement_range = RtdRange::PT100;
        m.ch_params[3].measurement_range = RtdRange::PT1000;

        assert_eq!(
            m.process_input_data(&vec![0x6C00, 0x7EFF, 55, 99]).unwrap(),
            vec![
                Decimal32(40.0),
                Decimal32(47.03559),
                Decimal32(5.5),
                Decimal32(9.9),
            ]
        );
    }

    #[test]
    fn test_process_input_data_with_negative_temperatures() {
        let mut m = Mod::default();
        m.ch_params[0].measurement_range = RtdRange::PT100;
        m.ch_params[1].measurement_range = RtdRange::Cu10;

        assert_eq!(
            m.process_input_data(&vec![0xF830, 0xFF38, 0, 0]).unwrap(),
            vec![Decimal32(-200.0), Decimal32(-20.0), Disabled, Disabled]
        );
    }

    #[test]
    fn test_process_output_values() {
        let m = Mod::default();
        assert!(
            m.process_output_values(&[ChannelValue::Decimal32(0.0)])
                .is_err()
        );
        assert_eq!(m.process_output_values(&[]).unwrap(), &[]);
    }
}
