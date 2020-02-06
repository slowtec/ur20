//! Digital frequency counter module UR20-2FCNT-100

use super::*;
use crate::ur20_fbc_mod_tcp::{FromModbusParameterData, ProcessModbusTcpData};
use num_traits::cast::FromPrimitive;
use std::time::Duration;

lazy_static! {
    static ref MAX_MEASUREMENT_DURATION: Duration = Duration::new(8, 388_607_000);
}

const MICROS_PER_SEC: u32 = 1_000_000;
const NANOS_PER_SEC: u32 = 1_000_000_000;
const MAX_MEASUREMENT_PERIOD: u64 = 0x07FF_FFFF;

#[derive(Debug, Clone)]
pub struct Mod {
    pub ch_params: Vec<ChannelParameters>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessInput {
    /// Current period duration
    pub duration: Option<Duration>,
    /// Number of rising edges within the current measurement cycle
    pub count: u32,
    /// Measurement active
    pub active: bool,
}

impl ProcessInput {
    /// Calculate the frequency in Hz.
    pub fn hertz(&self) -> Option<f32> {
        if let Some(d) = self.duration {
            //TODO: check overflow!
            Some(
                self.count as f32
                    / (d.as_secs() as f32 + d.subsec_nanos() as f32 / NANOS_PER_SEC as f32),
            )
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Measurement command
pub enum Command {
    /// Measurement start
    Start,
    /// Measurement stop
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProcessOutput {
    /// Preset value of the measurement cycle period
    pub duration: Duration,
    /// Command to start or stop the measurement
    pub command: Option<Command>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelParameters {
    /// Signal filter
    pub input_filter: InputFilter,
}

impl Default for ChannelParameters {
    fn default() -> Self {
        ChannelParameters {
            input_filter: InputFilter::us5,
        }
    }
}

impl Default for ProcessOutput {
    fn default() -> Self {
        ProcessOutput {
            duration: Duration::new(0, 0),
            command: None,
        }
    }
}

impl From<ProcessInput> for ChannelValue {
    fn from(i: ProcessInput) -> Self {
        ChannelValue::FcntIn(i)
    }
}

impl From<ProcessOutput> for ChannelValue {
    fn from(o: ProcessOutput) -> Self {
        ChannelValue::FcntOut(o)
    }
}

impl Default for Mod {
    fn default() -> Self {
        let ch_params = (0..2).map(|_| ChannelParameters::default()).collect();
        Mod { ch_params }
    }
}

impl Module for Mod {
    fn module_type(&self) -> ModuleType {
        ModuleType::UR20_2FCNT_100
    }
}

impl FromModbusParameterData for Mod {
    fn from_modbus_parameter_data(data: &[u16]) -> Result<Mod> {
        let ch_params = parameters_from_raw_data(data)?;
        Ok(Mod { ch_params })
    }
}

impl ProcessModbusTcpData for Mod {
    fn process_input_byte_count(&self) -> usize {
        20
    }
    fn process_output_byte_count(&self) -> usize {
        12
    }
    fn process_input_data(&self, data: &[u16]) -> Result<Vec<ChannelValue>> {
        if data.len() != 10 {
            return Err(Error::BufferLength);
        }

        if self.ch_params.len() != 2 {
            return Err(Error::ChannelParameter);
        }

        let res = (0..2)
            .map(|i| {
                let idx = i * 4;
                (&data[idx..idx + 2], &data[idx + 2..idx + 4], &data[8 + i])
            })
            .map(|(duration, cnt, active)| {
                (
                    {
                        let d = ((duration[0] as u32) << 16 | duration[1] as u32) as u64;
                        if d >= MAX_MEASUREMENT_PERIOD {
                            None
                        } else {
                            Some(Duration::from_nanos(d * 125))
                        }
                    },
                    ((cnt[0] as u32) << 16 | cnt[1] as u32),
                    util::test_bit_16(*active, 8),
                )
            })
            .map(|(duration, count, active)| {
                ChannelValue::FcntIn(ProcessInput {
                    duration,
                    count,
                    active,
                })
            })
            .collect();
        Ok(res)
    }
    fn process_output_data(&self, data: &[u16]) -> Result<Vec<ChannelValue>> {
        if data.len() != 6 {
            return Err(Error::BufferLength);
        }

        let res = (0..2)
            .map(|i| {
                let idx = i * 2;
                (&data[idx..idx + 2], &data[4 + i])
            })
            .map(|(duration, control)| {
                let cmd = if util::test_bit_16(*control, 8) {
                    Some(Command::Start)
                } else if util::test_bit_16(*control, 9) {
                    Some(Command::Stop)
                } else {
                    None
                };
                (
                    ((duration[0] as u32) << 16 | duration[1] as u32) as u64,
                    cmd,
                )
            })
            .map(|(duration, command)| {
                ChannelValue::FcntOut(ProcessOutput {
                    duration: Duration::from_nanos(duration * 1000),
                    command,
                })
            })
            .collect();
        Ok(res)
    }
    fn process_output_values(&self, values: &[ChannelValue]) -> Result<Vec<u16>> {
        let cnt = self.module_type().channel_count();
        if values.len() != cnt {
            return Err(Error::ChannelValue);
        }
        if self.ch_params.len() != cnt {
            return Err(Error::ChannelParameter);
        }
        let mut out = vec![0; 6];

        for (i, v) in values.iter().enumerate() {
            match v {
                ChannelValue::FcntOut(v) => {
                    if v.duration > *MAX_MEASUREMENT_DURATION {
                        return Err(Error::ChannelValue);
                    }
                    let micros =
                        v.duration.as_secs() as u32 * MICROS_PER_SEC + v.duration.subsec_micros();
                    let lo = micros & 0x0000_FFFF;
                    let hi = (micros & 0xFFFF_0000) >> 16;
                    let idx = i * 2;
                    out[idx] = hi as u16;
                    out[idx + 1] = lo as u16;
                    if let Some(cmd) = v.command {
                        let idx = i + 4;
                        match cmd {
                            Command::Start => {
                                out[idx] = util::set_bit_16(0, 8);
                            }
                            Command::Stop => {
                                out[idx] = util::set_bit_16(0, 9);
                            }
                        }
                    }
                }
                ChannelValue::Disabled => { /* ignore */ }
                _ => {
                    return Err(Error::ChannelValue);
                }
            }
        }
        Ok(out)
    }
}

fn parameters_from_raw_data(data: &[u16]) -> Result<Vec<ChannelParameters>> {
    if data.len() < 2 {
        return Err(Error::BufferLength);
    }

    let channel_parameters: Result<Vec<_>> = (0..2)
        .map(|idx| {
            let mut p = ChannelParameters::default();

            p.input_filter = match FromPrimitive::from_u16(data[idx]) {
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

    #[test]
    fn test_channel_parameters_from_raw_data() {
        assert_eq!(parameters_from_raw_data(&[0, 0]).unwrap().len(), 2);
        assert_eq!(
            parameters_from_raw_data(&[0, 0]).unwrap(),
            vec![ChannelParameters::default(); 2]
        );
        assert_eq!(
            parameters_from_raw_data(&[0, 1]).unwrap()[1].input_filter,
            InputFilter::us11
        );
        assert_eq!(
            parameters_from_raw_data(&[2, 1]).unwrap()[0].input_filter,
            InputFilter::us21
        );
    }

    #[test]
    fn test_parameters_from_invalid_data_buffer_size() {
        assert!(parameters_from_raw_data(&[0; 0]).is_err());
        assert!(parameters_from_raw_data(&[0; 1]).is_err());
        assert!(parameters_from_raw_data(&[0; 2]).is_ok());
    }

    #[test]
    fn process_input_byte_count() {
        let m = Mod::default();
        assert_eq!(m.process_input_byte_count(), 20);
    }

    #[test]
    fn process_output_byte_count() {
        let m = Mod::default();
        assert_eq!(m.process_output_byte_count(), 12);
    }

    #[test]
    fn test_process_input_data_with_invalid_buffer_size() {
        let m = Mod::default();
        assert!(m.process_input_data(&vec![]).is_err());
        assert!(m.process_input_data(&vec![0; 2]).is_err());
        assert!(m.process_input_data(&vec![0; 9]).is_err());
        assert!(m.process_input_data(&vec![0; 10]).is_ok());
    }

    #[test]
    fn test_process_input_data_with_missing_channel_parameters() {
        let mut m = Mod::default();
        m.ch_params = vec![];
        assert!(m.process_input_data(&vec![0; 10]).is_err());
    }

    #[test]
    fn test_process_input_data() {
        let m = Mod::default();
        let mut data = vec![
            0, 0, // channel 0 - duration
            0, 0, // channel 0 - count
            0, 0, // channel 1 - duration
            0, 0, // channel 1 - count
            0, // channel 0 - active
            0, // chanel  1 - active
        ];

        let res = m.process_input_data(&data).unwrap();
        let inactive = ChannelValue::FcntIn(ProcessInput {
            count: 0,
            active: false,
            duration: Some(Duration::new(0, 0)),
        });
        assert_eq!(res[0], inactive);
        assert_eq!(res[1], inactive);

        data[1] = 1200;
        data[3] = 3;
        data[8] = util::set_bit_16(0, 8);
        let active = ChannelValue::FcntIn(ProcessInput {
            count: 3,
            active: true,
            duration: Some(Duration::from_micros(150)),
        });
        let res = m.process_input_data(&data).unwrap();
        assert_eq!(res[0], active);
        assert_eq!(res[1], inactive);
    }

    #[test]
    fn test_process_input_data_min_duration() {
        let m = Mod::default();
        let mut data = vec![0; 10];
        // 1µs = 0x0000_0008
        data[1] = 0x8;
        let expected = ChannelValue::FcntIn(ProcessInput {
            count: 0,
            active: false,
            duration: Some(Duration::from_micros(1)),
        });
        assert_eq!(m.process_input_data(&data).unwrap()[0], expected);
    }

    #[test]
    fn test_process_input_data_max_duration() {
        let m = Mod::default();
        let mut data = vec![0; 10];
        data[0] = 0x07FF;
        data[1] = 0xFFFE;
        data[4] = 0x07FF;
        data[5] = 0xFFFF;
        let expected_0 = ChannelValue::FcntIn(ProcessInput {
            count: 0,
            active: false,
            duration: Some(Duration::from_nanos((0x07FF_FFFF - 1) * 125)),
        });
        let expected_1 = ChannelValue::FcntIn(ProcessInput {
            count: 0,
            active: false,
            duration: None,
        });
        assert_eq!(m.process_input_data(&data).unwrap()[0], expected_0);
        assert_eq!(m.process_input_data(&data).unwrap()[1], expected_1);
    }

    #[test]
    fn test_process_output_data_with_invalid_buffer_size() {
        let m = Mod::default();
        assert!(m.process_output_data(&[]).is_err());
        assert!(m.process_output_data(&[0; 5]).is_err());
        assert!(m.process_output_data(&[0; 7]).is_err());
        assert!(m.process_output_data(&[0; 6]).is_ok());
    }

    #[test]
    fn test_process_output_data() {
        let m = Mod::default();
        let mut data = vec![
            0, 0, // channel 0 - measurement time
            0, 0, // channel 1 - measurement time
            0, // channel 0 - control
            0, // chanel  1 - control
        ];
        let res = m.process_output_data(&data).unwrap();
        let inactive = ChannelValue::FcntOut(ProcessOutput {
            duration: Duration::new(0, 0),
            command: None,
        });
        assert_eq!(res[0], inactive);
        assert_eq!(res[1], inactive);

        data[1] = 120; // 120 µs
        data[3] = 3;

        let dur_120 = ChannelValue::FcntOut(ProcessOutput {
            duration: Duration::new(0, 120000),
            command: None,
        });
        let dur_3 = ChannelValue::FcntOut(ProcessOutput {
            duration: Duration::new(0, 3000),
            command: None,
        });

        let res = m.process_output_data(&data).unwrap();
        assert_eq!(res[0], dur_120);
        assert_eq!(res[1], dur_3);

        let start = ChannelValue::FcntOut(ProcessOutput {
            duration: Duration::new(0, 0),
            command: Some(Command::Start),
        });

        let stop = ChannelValue::FcntOut(ProcessOutput {
            duration: Duration::new(0, 0),
            command: Some(Command::Stop),
        });

        data[1] = 0;
        data[3] = 0;
        data[4] = util::set_bit_16(0, 8);
        data[5] = util::set_bit_16(0, 9);

        let res = m.process_output_data(&data).unwrap();
        assert_eq!(res[0], start);
        assert_eq!(res[1], stop);
    }

    #[test]
    fn test_process_output_values_with_invalid_channel_len() {
        let m = Mod::default();
        let out = ProcessOutput::default();
        assert!(m.process_output_values(&[]).is_err());
        assert!(m.process_output_values(&vec![out.into(); 1]).is_err());
        assert!(m.process_output_values(&vec![out.into(); 3]).is_err());
        assert!(m.process_output_values(&vec![out.into(); 2]).is_ok());
    }

    #[test]
    fn test_process_output_values_with_missing_channel_parameters() {
        let mut m = Mod::default();
        m.ch_params = vec![];
        let out = ProcessOutput::default();
        assert!(m.process_output_values(&vec![out.into(); 2]).is_err());
    }

    #[test]
    fn test_process_output_values_with_invalid_channel_values() {
        let m = Mod::default();
        assert!(m
            .process_output_values(&[ChannelValue::Bit(false), ChannelValue::Decimal32(0.0),])
            .is_err());
    }

    #[test]
    fn test_process_output_values() {
        let m = Mod::default();
        let ch_0 = ProcessOutput::default();
        let ch_1 = ProcessOutput::default();

        assert_eq!(
            m.process_output_values(&[ch_0.into(), ch_1.into()])
                .unwrap(),
            vec![0; 6]
        );

        let mut ch_0 = ProcessOutput::default();
        let mut ch_1 = ProcessOutput::default();
        ch_0.duration = Duration::new(2, 0);
        ch_1.duration = Duration::new(0, 1_000);

        assert_eq!(
            m.process_output_values(&[ch_0.into(), ch_1.into()])
                .unwrap(),
            vec![0x1e, 0x8480, 0, 1, 0, 0]
        );

        let mut ch_0 = ProcessOutput::default();
        let mut ch_1 = ProcessOutput::default();
        ch_0.duration = Duration::new(0, 1_000);
        ch_0.command = Some(Command::Start);
        ch_1.duration = Duration::new(8, 388_607_000);
        ch_1.command = Some(Command::Stop);

        assert_eq!(
            m.process_output_values(&[ch_0.into(), ch_1.into()])
                .unwrap(),
            vec![0, 1, 0b0111_1111, 0xFFFF, 0x01_00, 0x02_00]
        );
    }

    #[test]
    fn test_process_output_values_with_invalid_duration() {
        let m = Mod::default();
        let mut ch_0 = ProcessOutput::default();
        let mut ch_1 = ProcessOutput::default();
        ch_0.duration = Duration::new(0, 1_000);
        ch_1.duration = Duration::new(8, 388_608_000);
        assert!(m
            .process_output_values(&[ch_0.into(), ch_1.into()])
            .is_err());
    }

    #[test]
    fn test_process_input_hertz() {
        let input = ProcessInput {
            count: 100,
            active: true,
            duration: Some(Duration::new(1, 0)),
        };
        assert_eq!(input.hertz().unwrap(), 100.0);
        let input = ProcessInput {
            count: 5,
            active: true,
            duration: Some(Duration::new(0, 200_000)),
        };
        assert_eq!(input.hertz().unwrap(), 25000.0);
        let input = ProcessInput {
            count: ::std::u32::MAX,
            active: true,
            duration: Some(Duration::new(0, 1_000)),
        };
        assert_eq!(input.hertz().unwrap(), 4_294_967_295_000_000.0);
        let input = ProcessInput {
            count: 5,
            active: true,
            duration: None,
        };
        assert_eq!(input.hertz(), None);
    }
}
