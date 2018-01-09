//! Analog output module UR20-4AO-UI-16

use super::*;

#[derive(Debug)]
pub struct Mod {
    pub ch_params: Vec<ChannelParameters>,
}

#[derive(Debug, Clone)]
pub struct ChannelParameters {
    pub data_format: DataFormat,
    pub output_range: AnalogUIRange,
    pub substitute_value: f32,
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
    fn process_input_byte_count(&self) -> usize {
        0
    }
    fn process_output_byte_count(&self) -> usize {
        8
    }
    fn module_type(&self) -> ModuleType {
        ModuleType::UR20_4AO_UI_16
    }
    fn process_input_data(&mut self, _: &[u16]) -> Result<Vec<ChannelValue>> {
        Ok((0..4).map(|_| ChannelValue::None).collect())
    }
    fn process_output_values(&mut self, values: &[ChannelValue]) -> Result<Vec<u16>> {
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
                    match self.ch_params[i].data_format {
                        DataFormat::S5 => S5_FACTOR,
                        DataFormat::S7 => S7_FACTOR,
                    } as f32,
                )
            })
            .map(|(v, range, factor)| match *v {
                ChannelValue::Decimal32(v) => {

                    use AnalogUIRange::*;

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
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use ChannelValue::*;

    #[test]
    fn test_process_output_values_with_invalid_channel_len() {
        let mut m = Mod::default();
        assert!(m.process_output_values(&[]).is_err());
        assert!(
            m.process_output_values(&[Decimal32(0.0), Decimal32(0.0), Decimal32(0.0)])
                .is_err()
        );
        assert!(
            m.process_output_values(
                &[
                    Decimal32(0.0),
                    Decimal32(0.0),
                    Decimal32(0.0),
                    Decimal32(0.0),
                ],
            ).is_ok()
        );
    }

    #[test]
    fn test_process_output_values_with_invalid_channel_values() {
        let mut m = Mod::default();
        assert!(
            m.process_output_values(
                &[Bit(false), Decimal32(0.0), Decimal32(0.0), Decimal32(0.0)],
            ).is_err()
        );
    }

    #[test]
    fn test_process_output_values_with_missing_channel_parameters() {
        let mut m = Mod::default();
        m.ch_params = vec![];
        assert!(
            m.process_output_values(
                &[
                    Decimal32(0.0),
                    Decimal32(0.0),
                    Decimal32(0.0),
                    Decimal32(0.0),
                ],
            ).is_err()
        );
    }

    #[test]
    fn test_process_output_values() {
        let mut m = Mod::default();
        assert_eq!(
            m.process_output_values(
                &[
                    Decimal32(0.0),
                    Decimal32(99.9),
                    Decimal32(0.0),
                    Decimal32(3.3),
                ],
            ).unwrap(),
            vec![0, 0, 0, 0]
        );
        m.ch_params[0].output_range = AnalogUIRange::mA0To20;
        m.ch_params[1].output_range = AnalogUIRange::mA0To20;
        m.ch_params[2].output_range = AnalogUIRange::mA0To20;
        m.ch_params[3].output_range = AnalogUIRange::mA0To20;
        assert_eq!(
            m.process_output_values(
                &[
                    Decimal32(23.518),
                    Decimal32(20.0),
                    Decimal32(10.0),
                    Decimal32(0.0),
                ],
            ).unwrap(),
            vec![0x7EFF, 0x6C00, 0x3600, 0x0]
        );

        m.ch_params[0].output_range = AnalogUIRange::mA0To20;
        m.ch_params[1].output_range = AnalogUIRange::mA4To20;
        m.ch_params[2].output_range = AnalogUIRange::V0To10;
        m.ch_params[3].output_range = AnalogUIRange::VMinus10To10;
        assert_eq!(
            m.process_output_values(
                &[
                    Decimal32(10.0),
                    Decimal32(12.0),
                    Decimal32(10.0),
                    Decimal32(-10.0),
                ],
            ).unwrap(),
            vec![0x3600, 0x3600, 0x6C00, 0x9400]
        );

        m.ch_params[0].output_range = AnalogUIRange::V0To5;
        m.ch_params[1].output_range = AnalogUIRange::VMinus5To5;
        m.ch_params[2].output_range = AnalogUIRange::V1To5;
        m.ch_params[3].output_range = AnalogUIRange::V2To10;
        assert_eq!(
            m.process_output_values(
                &[
                    Decimal32(5.0),
                    Decimal32(-5.0),
                    Decimal32(1.0),
                    Decimal32(2.0),
                ],
            ).unwrap(),
            vec![0x6C00, 0x9400, 0x0, 0x0]
        );
    }
}
