//! Modbus TCP fieldbus coupler UR20-FBC-MOD-TCP

use super::*;
use util::*;

type Word = u16;
type RegisterAddress = u16;
type BitAddress = u16;
type BitNr = usize;

pub const ADDR_PACKED_PROCESS_INPUT_DATA  : RegisterAddress = 0x0000;
pub const ADDR_PACKED_PROCESS_OUTPUT_DATA : RegisterAddress = 0x0800;
pub const ADDR_PROCESS_OUTPUT_LEN         : RegisterAddress = 0x1010;
pub const ADDR_PROCESS_INPUT_LEN          : RegisterAddress = 0x1011;
pub const ADDR_COUPLER_ID                 : RegisterAddress = 0x1000;
pub const ADDR_COUPLER_STATUS             : RegisterAddress = 0x100C;
pub const ADDR_CURRENT_MODULE_COUNT       : RegisterAddress = 0x27FE;
pub const ADDR_CURRENT_MODULE_LIST        : RegisterAddress = 0x2A00;
pub const ADDR_MODULE_OFFSETS             : RegisterAddress = 0x2B00;
pub const ADDR_MODULE_PARAMETERS          : RegisterAddress = 0xC000;

/// The packed process data offset addresses of a module.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleOffset {
    pub input: Option<BitAddress>,
    pub output: Option<BitAddress>,
}

/// Converts the register data into a list of module offsets.
pub fn offsets_of_process_data(data: &[Word]) -> Vec<ModuleOffset> {
    let mut offsets = vec![];
    for i in 0..data.len() / 2 {
        offsets.push(ModuleOffset {
            input: word_to_offset(data[i * 2 + 1]),
            output: word_to_offset(data[i * 2]),
        });
    }
    offsets
}

/// Map the raw data into values.
pub fn process_input_data(
    modules: &mut [(Box<Module>, ModuleOffset)],
    data: &[u16],
) -> Result<Vec<Vec<ChannelValue>>> {

    modules
        .into_iter()
        .map(|&mut (ref mut m, ref offset)| {

            if let Some(in_offset) = offset.input {

                let (start, bit) = to_register_address(in_offset);
                let mut start = (start - ADDR_PACKED_PROCESS_INPUT_DATA) as usize;
                let word_count = {
                    let cnt = m.process_input_byte_count() / 2;
                    if cnt == 0 { 1 } else { cnt }
                };
                let end = start + word_count;
                if end > data.len() {
                    return Err(Error::BufferLength);
                }
                let input = &data[start..end];

                match bit {
                    0 => m.process_input_data(input),
                    8 => {
                        let buf = u16_to_u8(input);
                        let buf = &buf[1..]; // drop first byte
                        let mut shifted = vec![];
                        shifted.extend_from_slice(buf);
                        shifted.push(0);
                        m.process_input_data(&u8_to_u16(&shifted))
                    }
                    _ => Err(Error::ModuleOffset),
                }

            } else {
                Ok(vec![])
            }
        })
        .collect()
}

/// Map values into raw values.
pub fn process_output_values(
    modules: &mut [(Box<Module>, ModuleOffset)],
    values: &[Vec<ChannelValue>]
) -> Result<Vec<u16>> {

    if modules.len() != values.len() {
        return Err(Error::ChannelValue);
    }

    let mut out = vec![];

    for (i, &mut (ref mut m, ref offset)) in modules.into_iter().enumerate() {

         if let Some(out_offset) = offset.output {

             let data = m.process_output_values(&values[i])?;
             let (start, bit) = to_register_address(out_offset);
             if start < ADDR_PACKED_PROCESS_OUTPUT_DATA {
                return Err(Error::ModuleOffset);
             }
             let start = (start - ADDR_PACKED_PROCESS_OUTPUT_DATA) as usize;

             match bit {
                 0 => {
                    if out.len() != start {
                       return Err(Error::ModuleOffset);
                    }
                    out.extend_from_slice(&data);
                 }
                 8 => {
                     if out.len() != start + 1 {
                        return Err(Error::ModuleOffset);
                     }
                     let shared_low_byte = out[start as usize] & 0x00FF;
                     let buf = u16_to_u8(&data);
                     let shared_high_byte =  (buf[0] as u16) << 8;
                     let word = shared_high_byte | shared_low_byte;
                     out[start as usize] = word;
                 }
                 _ => {
                     return Err(Error::ModuleOffset);
                 }
             }
         }
    }

    Ok(out)
}

fn word_to_offset(word: Word) -> Option<BitAddress> {
    if word == 0xFFFF { None } else { Some(word) }
}

/// Splits a bit address into a register address and a bit number.
pub fn to_register_address(addr: BitAddress) -> (RegisterAddress, BitNr) {
    let register = (addr & 0xFFF0) >> 4;
    let bit = (addr & 0x000F) as usize;
    (register as u16, bit)
}

/// Merges a register address and a bit number into a bit address.
pub fn to_bit_address(addr: RegisterAddress, bit: usize) -> BitAddress {
    (addr << 4) | (bit as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offsets_of_process_data() {
        assert_eq!(offsets_of_process_data(&vec![]), vec![]);
        assert_eq!(
            offsets_of_process_data(&vec![0xFFFF, 0x0000, 0x8000, 0x0040, 0x8050, 0xFFFF]),
            vec![
                ModuleOffset {
                    output: None,
                    input: Some(0x0000),
                },
                ModuleOffset {
                    output: Some(0x8000),
                    input: Some(0x0040),
                },
                ModuleOffset {
                    output: Some(0x8050),
                    input: None,
                },
            ]
        );
    }

    #[test]
    fn test_to_regsiter_address() {
        assert_eq!(to_register_address(0x80AB), (0x080A, 11));
    }

    #[test]
    fn test_to_bit_address() {
        assert_eq!(to_bit_address(0x080A, 11), 0x080AB);
    }

    #[test]
    fn test_process_input_data() {

        let m0     = super::ur20_4ao_ui_16::Mod::default();
        let mut m1 = super::ur20_4ai_rtd_diag::Mod::default();
        let m2     = super::ur20_4di_p::Mod::default();
        let m3     = super::ur20_4di_p::Mod::default();

        let data = &[
            0,33,0,0,             // UR20-4AI-P
            0b0000_0001_0000_0010 // UR20-4DI-P + UR20-4DI-P
        ];

        m1.ch_params[1].measurement_range = RtdRange::PT100;

        let mod0: Box<Module> = Box::new(m0);
        let mod1: Box<Module> = Box::new(m1);
        let mod2: Box<Module> = Box::new(m2);
        let mod3: Box<Module> = Box::new(m3);

        let addr_out_0 = to_bit_address(ADDR_PACKED_PROCESS_OUTPUT_DATA, 0);
        let addr_in_1 = to_bit_address(ADDR_PACKED_PROCESS_INPUT_DATA, 0);
        let addr_in_2 = to_bit_address(ADDR_PACKED_PROCESS_INPUT_DATA + 4, 0);
        let addr_in_3 = to_bit_address(ADDR_PACKED_PROCESS_INPUT_DATA + 4, 8);

        let o0 = ModuleOffset {
            input: None,
            output: Some(addr_out_0),
        };
        let o1 = ModuleOffset {
            input: Some(addr_in_1),
            output: None,
        };
        let o2 = ModuleOffset {
            input: Some(addr_in_2),
            output: None,
        };
        let o3 = ModuleOffset {
            input: Some(addr_in_3),
            output: None,
        };

        let mut modules = vec![(mod0, o0), (mod1, o1), (mod2, o2), (mod3, o3)];

        let res = process_input_data(&mut modules, data).unwrap();
        assert_eq!(res.len(), 4);
        assert_eq!(res[0].len(), 0);
        assert_eq!(res[1].len(), 4);
        assert_eq!(res[2].len(), 4);
        assert_eq!(res[3].len(), 4);
        assert_eq!(res[1][1], ChannelValue::Decimal32(3.3));
        assert_eq!(res[2][1], ChannelValue::Bit(true));
        assert_eq!(res[3][0], ChannelValue::Bit(true));

    }

    #[test]
    fn test_process_input_data_with_invalid_offset() {
        let m0 = super::ur20_4ai_rtd_diag::Mod::default();
        let data = &[0, 33, 0, 0];
        let mod0: Box<Module> = Box::new(m0);
        let bit = 3; // should not work
        let addr_in_0 = to_bit_address(ADDR_PACKED_PROCESS_INPUT_DATA, bit);
        let o0 = ModuleOffset {
            input: Some(addr_in_0),
            output: None,
        };
        let mut modules = vec![(mod0, o0)];
        assert!(process_input_data(&mut modules, data).is_err());
    }

    #[test]
    fn test_process_input_data_with_invalid_data() {
        let m0 = super::ur20_4ai_rtd_diag::Mod::default();
        let m1 = super::ur20_4ai_rtd_diag::Mod::default();
        let data = &[0, 33, 0, 0];
        let mod0: Box<Module> = Box::new(m0);
        let mod1: Box<Module> = Box::new(m1);
        let addr_in_0 = to_bit_address(ADDR_PACKED_PROCESS_INPUT_DATA, 0);
        let addr_in_1 = to_bit_address(ADDR_PACKED_PROCESS_INPUT_DATA + 4, 0);
        let o0 = ModuleOffset {
            input: Some(addr_in_0),
            output: None,
        };
        let o1 = ModuleOffset {
            input: Some(addr_in_1),
            output: None,
        };
        let mut modules = vec![(mod0, o0), (mod1, o1)];
        assert!(process_input_data(&mut modules, data).is_err());
    }

    #[test]
    fn test_process_output_values_with_invalid_len() {

        let m0 = super::ur20_4ao_ui_16::Mod::default();
        let m1 = super::ur20_4ai_rtd_diag::Mod::default();

        let values = vec![
            vec![
                ChannelValue::Decimal32(15.0),
                ChannelValue::Decimal32(20.0),
                ChannelValue::Decimal32(20.0),
                ChannelValue::Decimal32(10.0),
            ],
        ];

        let mod0: Box<Module> = Box::new(m0);
        let mod1: Box<Module> = Box::new(m1);

        let addr_out_0 = to_bit_address(ADDR_PACKED_PROCESS_OUTPUT_DATA, 0);
        let addr_in_1  = to_bit_address(ADDR_PACKED_PROCESS_INPUT_DATA, 0);

        let o0 = ModuleOffset {
            input: None,
            output: Some(addr_out_0),
        };
        let o1 = ModuleOffset {
            input: Some(addr_in_1),
            output: None,
        };

        let mut modules = vec![(mod0, o0), (mod1, o1)];

        assert!(process_output_values(&mut modules, &values).is_err());
    }

    #[test]
    fn test_process_output_values_with_invalid_offset_a() {

        let m0 = super::ur20_4ao_ui_16::Mod::default();
        let m1 = super::ur20_4ai_rtd_diag::Mod::default();

        let values = vec![
            vec![
                ChannelValue::Decimal32(15.0),
                ChannelValue::Decimal32(20.0),
                ChannelValue::Decimal32(20.0),
                ChannelValue::Decimal32(10.0),
            ],
            vec![]
        ];

        let mod0: Box<Module> = Box::new(m0);
        let mod1: Box<Module> = Box::new(m1);

        let addr_out_0 = to_bit_address(ADDR_PACKED_PROCESS_OUTPUT_DATA + 10, 0);
        let addr_in_1  = to_bit_address(ADDR_PACKED_PROCESS_INPUT_DATA, 0);

        let o0 = ModuleOffset {
            input: None,
            output: Some(addr_out_0),
        };
        let o1 = ModuleOffset {
            input: Some(addr_in_1),
            output: None,
        };

        let mut modules = vec![(mod0, o0), (mod1, o1)];
        assert!(process_output_values(&mut modules, &values).is_err());
    }

    #[test]
    fn test_process_output_values_with_invalid_offset_b() {

        let m0 = super::ur20_4ao_ui_16::Mod::default();
        let m1 = super::ur20_4ai_rtd_diag::Mod::default();
        let m2 = super::ur20_4do_p::Mod::default();

        let values = vec![
            vec![
                ChannelValue::Decimal32(15.0),
                ChannelValue::Decimal32(20.0),
                ChannelValue::Decimal32(20.0),
                ChannelValue::Decimal32(10.0),
            ],
            vec![],
            vec![
                ChannelValue::Bit(false),
                ChannelValue::Bit(false),
                ChannelValue::Bit(false),
                ChannelValue::Bit(false),
            ],
        ];

        let mod0: Box<Module> = Box::new(m0);
        let mod1: Box<Module> = Box::new(m1);
        let mod2: Box<Module> = Box::new(m2);

        let addr_out_0 = to_bit_address(ADDR_PACKED_PROCESS_OUTPUT_DATA + 0, 0);
        let addr_in_1  = to_bit_address(ADDR_PACKED_PROCESS_INPUT_DATA, 0);
        let addr_out_2 = to_bit_address(ADDR_PACKED_PROCESS_OUTPUT_DATA + 1, 8);

        let o0 = ModuleOffset {
            input: None,
            output: Some(addr_out_0),
        };
        let o1 = ModuleOffset {
            input: Some(addr_in_1),
            output: None,
        };
        let o2 = ModuleOffset {
            input: None,
            output: Some(addr_out_2),
        };

        let mut modules = vec![(mod0, o0), (mod1, o1), (mod2, o2)];
        assert!(process_output_values(&mut modules, &values).is_err());
    }

    #[test]
    fn test_process_output_values_with_invalid_offset_c() {
        let m0 = super::ur20_4ao_ui_16::Mod::default();
        let values = vec![
            vec![
                ChannelValue::Decimal32(15.0),
                ChannelValue::Decimal32(20.0),
                ChannelValue::Decimal32(20.0),
                ChannelValue::Decimal32(10.0),
            ],
        ];
        let mod0: Box<Module> = Box::new(m0);
        let addr_out_0 = to_bit_address(0, 0);
        let o0 = ModuleOffset {
            input: None,
            output: Some(addr_out_0),
        };
        let mut modules = vec![(mod0, o0)];
        assert!(process_output_values(&mut modules, &values).is_err());
    }

    #[test]
    fn test_process_output_values() {

        let mut m0 = super::ur20_4ao_ui_16::Mod::default();
        let m1     = super::ur20_4ai_rtd_diag::Mod::default();
        let m2     = super::ur20_4do_p::Mod::default();
        let m3     = super::ur20_4do_p::Mod::default();

        let values = vec![
            vec![
                ChannelValue::Decimal32(15.0),
                ChannelValue::Decimal32(20.0),
                ChannelValue::Decimal32(20.0),
                ChannelValue::Decimal32(10.0),
            ],
            vec![],
            vec![
                ChannelValue::Bit(false),
                ChannelValue::Bit(true),
                ChannelValue::Bit(false),
                ChannelValue::Bit(false),
            ],
            vec![
                ChannelValue::Bit(false),
                ChannelValue::Bit(false),
                ChannelValue::Bit(true),
                ChannelValue::Bit(true),
            ],
        ];

        m0.ch_params[1].output_range = AnalogUIRange::mA0To20;
        m0.ch_params[3].output_range = AnalogUIRange::mA0To20;

        let mod0: Box<Module> = Box::new(m0);
        let mod1: Box<Module> = Box::new(m1);
        let mod2: Box<Module> = Box::new(m2);
        let mod3: Box<Module> = Box::new(m3);

        let addr_out_0 = to_bit_address(ADDR_PACKED_PROCESS_OUTPUT_DATA, 0);
        let addr_in_1  = to_bit_address(ADDR_PACKED_PROCESS_INPUT_DATA, 0);
        let addr_out_2 = to_bit_address(ADDR_PACKED_PROCESS_OUTPUT_DATA + 4, 0);
        let addr_out_3 = to_bit_address(ADDR_PACKED_PROCESS_OUTPUT_DATA + 4, 8);

        let o0 = ModuleOffset {
            input: None,
            output: Some(addr_out_0),
        };
        let o1 = ModuleOffset {
            input: Some(addr_in_1),
            output: None,
        };
        let o2 = ModuleOffset {
            input: None,
            output: Some(addr_out_2),
        };
        let o3 = ModuleOffset {
            input: None,
            output: Some(addr_out_3),
        };

        let mut modules = vec![(mod0, o0), (mod1, o1), (mod2, o2), (mod3, o3)];

        let res = process_output_values(&mut modules, &values).unwrap();
        assert_eq!(res.len(), 5);
        assert_eq!(res[0], 0x0); // channel is disabled
        assert_eq!(res[1], 0x6C00);
        assert_eq!(res[2], 0x0); // channel is disabled
        assert_eq!(res[3], 0x3600);
        assert_eq!(res[4], 0b_0000_1100_0000_0010);
    }
}
