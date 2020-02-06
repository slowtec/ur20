//! Modbus TCP fieldbus coupler UR20-FBC-MOD-TCP

use super::*;
use crate::util::*;
use std::{
    collections::HashMap,
    io::{Read, Write},
};

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

pub trait ProcessModbusTcpData: Module {
    /// Number of bytes within the process input data buffer.
    fn process_input_byte_count(&self) -> usize;
    /// Number of bytes within the process output data buffer.
    fn process_output_byte_count(&self) -> usize;
    /// Transform raw module input data into a list of channel values.
    fn process_input_data(&self, data: &[u16]) -> Result<Vec<ChannelValue>> {
        if !data.is_empty() {
            return Err(Error::BufferLength);
        }
        let channel_cnt = self.module_type().channel_count();
        Ok(vec![ChannelValue::None; channel_cnt])
    }
    /// Transform raw module output data into a list of channel values.
    fn process_output_data(&self, data: &[u16]) -> Result<Vec<ChannelValue>> {
        if !data.is_empty() {
            return Err(Error::BufferLength);
        }
        let channel_cnt = self.module_type().channel_count();
        Ok(vec![ChannelValue::None; channel_cnt])
    }
    /// Transform channel values into raw module output data.
    fn process_output_values(&self, values: &[ChannelValue]) -> Result<Vec<u16>> {
        if !values.is_empty() && values.len() != self.module_type().channel_count() {
            return Err(Error::ChannelValue);
        }
        Ok(vec![])
    }
}

pub trait FromModbusParameterData {
    /// Create a new module instance.
    fn from_modbus_parameter_data(data: &[u16]) -> Result<Self>
    where
        Self: Sized + ProcessModbusTcpData;
}

/// The packed process data offset addresses of a module.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleOffset {
    pub input: Option<BitAddress>,
    pub output: Option<BitAddress>,
}

/// Modbus TCP coupler implementation.
pub struct Coupler {
    /// cached input values
    in_values: Vec<Vec<ChannelValue>>,
    /// cached output values
    out_values: Vec<Vec<ChannelValue>>,
    /// buffer write requests
    write: HashMap<Address, ChannelValue>,
    /// stateless modules
    modules: Vec<Box<dyn ProcessModbusTcpData>>,
    /// data offsets
    offsets: Vec<ModuleOffset>,
    /// statefull message processors
    processors: HashMap<usize, ur20_1com_232_485_422::MessageProcessor>,
    /// Last transmission counter  state
    last_tx_cnt: usize,
}

/// Raw config data to create a coupler instance.
#[derive(Debug, Clone)]
pub struct CouplerConfig {
    /// Register content of `ADDR_CURRENT_MODULE_LIST`.
    /// Register count: 2 * number of modules
    pub modules: Vec<ModuleType>,
    /// Register content of `ADDR_MODULE_OFFSETS`.
    /// Register count: 2 * number of modules
    pub offsets: Vec<u16>,
    /// Register content of `ADDR_MODULE_PARAMETERS`.
    pub params: Vec<Vec<u16>>,
}

impl Coupler {
    pub fn new(cfg: &CouplerConfig) -> Result<Self> {
        cfg.validate()?;

        let offsets = offsets_of_process_data(&cfg.offsets);

        let mut modules = vec![];
        let mut processors = HashMap::new();
        for (i, m) in cfg.modules.iter().enumerate() {
            let param_data = &cfg.params[i];
            let x: Box<dyn ProcessModbusTcpData> = match *m {
                ModuleType::UR20_4DI_P => {
                    let m = ur20_4di_p::Mod::from_modbus_parameter_data(&param_data)?;
                    Box::new(m)
                }
                ModuleType::UR20_4DO_P => {
                    let m = ur20_4do_p::Mod::from_modbus_parameter_data(&param_data)?;
                    Box::new(m)
                }
                ModuleType::UR20_16DO_P => {
                    let m = ur20_16do_p::Mod::from_modbus_parameter_data(&param_data)?;
                    Box::new(m)
                }
                ModuleType::UR20_4RO_CO_255 => {
                    let m = ur20_4ro_co_255::Mod::from_modbus_parameter_data(&param_data)?;
                    Box::new(m)
                }
                ModuleType::UR20_4AO_UI_16 => {
                    let m = ur20_4ao_ui_16::Mod::from_modbus_parameter_data(&param_data)?;
                    Box::new(m)
                }
                ModuleType::UR20_4AO_UI_16_DIAG => {
                    let m = ur20_4ao_ui_16_diag::Mod::from_modbus_parameter_data(&param_data)?;
                    Box::new(m)
                }
                ModuleType::UR20_4AI_RTD_DIAG => {
                    let m = ur20_4ai_rtd_diag::Mod::from_modbus_parameter_data(&param_data)?;
                    Box::new(m)
                }
                ModuleType::UR20_4AI_UI_16_DIAG => {
                    let m = ur20_4ai_ui_16_diag::Mod::from_modbus_parameter_data(&param_data)?;
                    Box::new(m)
                }
                ModuleType::UR20_4AI_UI_12 => {
                    let m = ur20_4ai_ui_12::Mod::from_modbus_parameter_data(&param_data)?;
                    Box::new(m)
                }
                ModuleType::UR20_8AI_I_16_DIAG_HD => {
                    let m = ur20_8ai_i_16_diag_hd::Mod::from_modbus_parameter_data(&param_data)?;
                    Box::new(m)
                }
                ModuleType::UR20_2FCNT_100 => {
                    let m = ur20_2fcnt_100::Mod::from_modbus_parameter_data(&param_data)?;
                    Box::new(m)
                }
                ModuleType::UR20_1COM_232_485_422 => {
                    let m = ur20_1com_232_485_422::Mod::from_modbus_parameter_data(&param_data)?;
                    let processor = ur20_1com_232_485_422::MessageProcessor::new(
                        m.mod_params.process_data_len.clone(),
                    );
                    processors.insert(i, processor);
                    Box::new(m)
                }
                _ => {
                    panic!("{:?} is not supported", m);
                }
            };
            modules.push(x);
        }
        Ok(Coupler {
            in_values: vec![],
            out_values: vec![],
            write: HashMap::new(),
            last_tx_cnt: 0,
            modules,
            offsets,
            processors,
        })
    }

    fn is_valid_addr(&self, addr: &Address) -> bool {
        addr.module < self.modules.len()
            && addr.channel < self.modules[addr.module].module_type().channel_count()
    }

    /// Returns current coupler input state.
    pub fn inputs(&self) -> &Vec<Vec<ChannelValue>> {
        &self.in_values
    }

    /// Returns current coupler output state.
    pub fn outputs(&self) -> &Vec<Vec<ChannelValue>> {
        &self.out_values
    }

    /// Returns a reader to the underlying communication data buffer.
    pub fn reader(&mut self, module_nr: usize) -> Option<&mut dyn Read> {
        self.processors
            .get_mut(&module_nr)
            .map(|r| r as &mut dyn Read)
    }

    /// Returns a writer to the underlying communication data buffer.
    pub fn writer(&mut self, module_nr: usize) -> Option<&mut dyn Write> {
        self.processors
            .get_mut(&module_nr)
            .map(|r| r as &mut dyn Write)
    }

    pub fn set_output(&mut self, addr: &Address, value: ChannelValue) -> Result<()> {
        if !self.is_valid_addr(addr) {
            return Err(Error::Address);
        }
        self.write.insert(addr.clone(), value);
        Ok(())
    }

    pub fn next(&mut self, process_input: &[u16], process_output: &[u16]) -> Result<Vec<u16>> {
        let infos: Vec<_> = self
            .modules
            .iter()
            .zip(&self.offsets)
            .map(|(m, o)| (&**m, o))
            .collect();
        self.in_values = process_input_data(&*infos, process_input)?;
        self.out_values = process_output_data(&*infos, process_output)?;

        let mut next_out_values = self.out_values.clone();
        let mut in_bytes = HashMap::new();
        let mut out_bytes = HashMap::new();

        for (m_nr, (in_v, out_v)) in self.in_values.iter().zip(&self.out_values).enumerate() {
            if let Some(p) = self.processors.get_mut(&m_nr) {
                if let ChannelValue::ComRsIn(ref in_v) = in_v[0] {
                    if let ChannelValue::ComRsOut(ref out_v) = out_v[0] {
                        out_bytes.insert(m_nr, ChannelValue::None);
                        in_bytes.insert(m_nr, ChannelValue::None);

                        if !out_v.data.is_empty() && out_v.tx_cnt != self.last_tx_cnt {
                            out_bytes.insert(m_nr, ChannelValue::Bytes(out_v.data.clone()));
                        }
                        self.last_tx_cnt = out_v.tx_cnt;

                        if let Some(v) = self.write.remove(&Address {
                            module: m_nr,
                            channel: 0,
                        }) {
                            if let ChannelValue::Bytes(ref data) = v {
                                p.write_all(data)?;
                            }
                        }

                        let rs_out = p.next(in_v, out_v);
                        next_out_values[m_nr][0] = ChannelValue::ComRsOut(rs_out);

                        if in_v.data_available && !in_v.data.is_empty() {
                            in_bytes.insert(m_nr, ChannelValue::Bytes(in_v.data.clone()));
                        }
                    }
                }
            } else {
                for (i, _) in out_v.iter().enumerate() {
                    if let Some(v) = self.write.remove(&Address {
                        module: m_nr,
                        channel: i,
                    }) {
                        next_out_values[m_nr][i] = v;
                    }
                }
            }
        }
        for (m_nr, v) in in_bytes {
            self.in_values[m_nr][0] = v;
        }
        for (m_nr, v) in out_bytes {
            self.out_values[m_nr][0] = v;
        }
        process_output_values(&*infos, &next_out_values)
    }
}

impl CouplerConfig {
    fn validate(&self) -> Result<()> {
        if self.modules.len() != self.params.len() {
            return Err(Error::BufferLength);
        }
        if self.modules.len() * 2 != self.offsets.len() {
            return Err(Error::ModuleOffset);
        }
        Ok(())
    }
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

/// Map the raw input data into values.
pub fn process_input_data(
    modules: &[(&dyn ProcessModbusTcpData, &ModuleOffset)],
    data: &[u16],
) -> Result<Vec<Vec<ChannelValue>>> {
    modules
        .iter()
        .map(|&(ref m, ref offset)| {
            if let Some(in_offset) = offset.input {
                let cnt = m.process_input_byte_count();
                m.process_input_data(&prepare_raw_data_to_process(
                    in_offset,
                    ADDR_PACKED_PROCESS_INPUT_DATA,
                    cnt,
                    data,
                )?)
            } else {
                Ok(vec![ChannelValue::None; m.module_type().channel_count()])
            }
        })
        .collect()
}

/// Map the raw output data into values.
pub fn process_output_data(
    modules: &[(&dyn ProcessModbusTcpData, &ModuleOffset)],
    data: &[u16],
) -> Result<Vec<Vec<ChannelValue>>> {
    modules
        .iter()
        .map(|&(ref m, ref offset)| {
            if let Some(out_offset) = offset.output {
                let cnt = m.process_output_byte_count();
                m.process_output_data(&prepare_raw_data_to_process(
                    out_offset,
                    ADDR_PACKED_PROCESS_OUTPUT_DATA,
                    cnt,
                    data,
                )?)
            } else {
                Ok(vec![ChannelValue::None; m.module_type().channel_count()])
            }
        })
        .collect()
}

fn prepare_raw_data_to_process(
    offset: u16,
    addr: u16,
    byte_count: usize,
    data: &[u16],
) -> Result<Vec<u16>> {
    let (start, bit) = to_register_address(offset);
    let start = (start - addr) as usize;
    let word_count = {
        let cnt = byte_count / 2;
        if cnt == 0 {
            1
        } else {
            cnt
        }
    };
    let end = start + word_count;
    if end > data.len() {
        return Err(Error::BufferLength);
    }
    let output = &data[start..end];

    match bit {
        0 => Ok(output.to_vec()),
        8 => Ok(shift_data(&output)),
        _ => Err(Error::ModuleOffset),
    }
}

/// Map values into raw values.
pub fn process_output_values(
    modules: &[(&dyn ProcessModbusTcpData, &ModuleOffset)],
    values: &[Vec<ChannelValue>],
) -> Result<Vec<u16>> {
    if modules.len() != values.len() {
        return Err(Error::ChannelValue);
    }

    let mut out = vec![];

    for (i, &(ref m, ref offset)) in modules.iter().enumerate() {
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
                    let shared_high_byte = u16::from(buf[0]) << 8;
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
    if word == 0xFFFF {
        None
    } else {
        Some(word)
    }
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

pub trait ModbusParameterRegisterCount {
    /// Total number of Modbus registers of module parameters.
    fn param_register_count(&self) -> u16;
}

impl ModbusParameterRegisterCount for ModuleType {
    fn param_register_count(&self) -> u16 {
        use super::ModuleType::*;
        match *self {
            // Digital input modules
            UR20_4DI_P | UR20_4DI_P_3W => 0 + 4 * 1,
            UR20_8DI_P_2W | UR20_8DI_P_3W => 0 + 8 * 1,

            // Digital output modules
            UR20_4DO_P => 0 + 4 * 1,
            UR20_16DO_P => 0,
            UR20_4RO_CO_255 => 0 + 4 * 1,

            // Analogue input modules
            UR20_8AI_I_16_DIAG_HD => 1 + 8 * 4,
            UR20_4AI_UI_16_DIAG => 1 + 4 * 5,
            UR20_4AI_UI_12 => 1 + 4 * 2,

            // Analogue output modul
            UR20_4AO_UI_16 => 0 + 4 * 3,
            UR20_4AO_UI_16_DIAG => 0 + 4 * 4,

            // Analogue input modules DIAG
            UR20_4AI_RTD_DIAG => 1 + 4 * 7,

            // Counter modules
            UR20_2FCNT_100 => 0 + 2 * 1,

            // Communication modules
            UR20_1COM_232_485_422 => 10,

            // Not yet supported
            _ => {
                panic!("{:?} is not supported", self);
            }
        }
    }
}

/// Calculate the parameter addresses and the number of registers by a given list of modules.
pub fn param_addresses_and_register_counts(modules: &[ModuleType]) -> Vec<(u16, u16)> {
    modules
        .iter()
        .enumerate()
        .map(|(idx, m)| {
            (
                ADDR_MODULE_PARAMETERS + (idx * 256) as u16,
                m.param_register_count(),
            )
        })
        .collect()
}

/// Converts the raw coupler register data into a list of module types.
pub fn module_list_from_registers(registers: &[u16]) -> Result<Vec<ModuleType>> {
    if registers.is_empty() || registers.len() % 2 != 0 {
        return Err(Error::RegisterCount);
    }
    let mut list = vec![];
    for i in 0..registers.len() / 2 {
        let idx = i as usize;
        let hi = u32::from(registers[idx * 2]);
        let lo = u32::from(registers[idx * 2 + 1]);
        let id = (hi << 16) + lo;
        let m = ModuleType::try_from_u32(id)?;
        list.push(m);
    }
    Ok(list)
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
        let m0 = super::ur20_4ao_ui_16::Mod::default();
        let mut m1 = super::ur20_4ai_rtd_diag::Mod::default();
        let m2 = super::ur20_4di_p::Mod::default();
        let m3 = super::ur20_4di_p::Mod::default();

        #[rustfmt::skip]
        let data = &[
            0,33,0,0,             // UR20-4AI-P
            0b0000_0001_0000_0010 // UR20-4DI-P + UR20-4DI-P
        ];

        m1.ch_params[1].measurement_range = RtdRange::PT100;

        let mod0: &dyn ProcessModbusTcpData = &m0;
        let mod1: &dyn ProcessModbusTcpData = &m1;
        let mod2: &dyn ProcessModbusTcpData = &m2;
        let mod3: &dyn ProcessModbusTcpData = &m3;

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

        let modules = vec![(mod0, &o0), (mod1, &o1), (mod2, &o2), (mod3, &o3)];

        let res = process_input_data(&modules, data).unwrap();
        assert_eq!(res.len(), 4);
        assert_eq!(res[0].len(), 4);
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
        let mod0: &dyn ProcessModbusTcpData = &m0;
        let bit = 3; // should not work
        let addr_in_0 = to_bit_address(ADDR_PACKED_PROCESS_INPUT_DATA, bit);
        let o0 = ModuleOffset {
            input: Some(addr_in_0),
            output: None,
        };
        let modules = vec![(mod0, &o0)];
        assert!(process_input_data(&modules, data).is_err());
    }

    #[test]
    fn test_process_input_data_with_invalid_data() {
        let m0 = super::ur20_4ai_rtd_diag::Mod::default();
        let m1 = super::ur20_4ai_rtd_diag::Mod::default();
        let data = &[0, 33, 0, 0];
        let mod0: &dyn ProcessModbusTcpData = &m0;
        let mod1: &dyn ProcessModbusTcpData = &m1;
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
        let modules = vec![(mod0, &o0), (mod1, &o1)];
        assert!(process_input_data(&modules, data).is_err());
    }

    #[test]
    fn test_process_output_data() {
        let mut m0 = super::ur20_4ao_ui_16::Mod::default();
        let m1 = super::ur20_4ai_rtd_diag::Mod::default();
        let m2 = super::ur20_4do_p::Mod::default();
        let m3 = super::ur20_4do_p::Mod::default();

        #[rustfmt::skip]
        let data = &[
            0,0x3600,0,0,         // UR20-4AO-P
            0b0000_0001_0000_0010 // UR20-4DO-P + UR20-4DO-P
        ];

        m0.ch_params[1].output_range = AnalogUIRange::VMinus5To5;

        let mod0: &dyn ProcessModbusTcpData = &m0;
        let mod1: &dyn ProcessModbusTcpData = &m1;
        let mod2: &dyn ProcessModbusTcpData = &m2;
        let mod3: &dyn ProcessModbusTcpData = &m3;

        let addr_out_0 = to_bit_address(ADDR_PACKED_PROCESS_OUTPUT_DATA, 0);
        let addr_in_1 = to_bit_address(ADDR_PACKED_PROCESS_INPUT_DATA, 0);
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

        let modules = vec![(mod0, &o0), (mod1, &o1), (mod2, &o2), (mod3, &o3)];

        let res = process_output_data(&modules, data).unwrap();
        assert_eq!(res.len(), 4);
        assert_eq!(res[0].len(), 4);
        assert_eq!(res[1].len(), 4);
        assert_eq!(res[2].len(), 4);
        assert_eq!(res[3].len(), 4);
        assert_eq!(res[0][1], ChannelValue::Decimal32(2.5));
        assert_eq!(res[1][0], ChannelValue::None);
        assert_eq!(res[2][1], ChannelValue::Bit(true));
        assert_eq!(res[3][0], ChannelValue::Bit(true));
    }

    #[test]
    fn test_process_output_data_with_invalid_offset() {
        let m0 = super::ur20_4ao_ui_16::Mod::default();
        let data = &[0, 33, 0, 0];
        let mod0: &dyn ProcessModbusTcpData = &m0;
        let bit = 3; // should not work
        let addr_out_0 = to_bit_address(ADDR_PACKED_PROCESS_OUTPUT_DATA, bit);
        let o0 = ModuleOffset {
            input: None,
            output: Some(addr_out_0),
        };
        let modules = vec![(mod0, &o0)];
        assert!(process_output_data(&modules, data).is_err());
    }

    #[test]
    fn test_process_output_data_with_invalid_data() {
        let m0 = super::ur20_4ao_ui_16::Mod::default();
        let m1 = super::ur20_4ao_ui_16::Mod::default();
        let data = &[0, 33, 0, 0];
        let mod0: &dyn ProcessModbusTcpData = &m0;
        let mod1: &dyn ProcessModbusTcpData = &m1;
        let addr_out_0 = to_bit_address(ADDR_PACKED_PROCESS_OUTPUT_DATA, 0);
        let addr_out_1 = to_bit_address(ADDR_PACKED_PROCESS_OUTPUT_DATA + 4, 0);
        let o0 = ModuleOffset {
            input: None,
            output: Some(addr_out_0),
        };
        let o1 = ModuleOffset {
            input: None,
            output: Some(addr_out_1),
        };
        let modules = vec![(mod0, &o0), (mod1, &o1)];
        assert!(process_output_data(&modules, data).is_err());
    }

    #[test]
    fn test_process_output_values_with_invalid_len() {
        let m0 = super::ur20_4ao_ui_16::Mod::default();
        let m1 = super::ur20_4ai_rtd_diag::Mod::default();

        let values = vec![vec![
            ChannelValue::Decimal32(15.0),
            ChannelValue::Decimal32(20.0),
            ChannelValue::Decimal32(20.0),
            ChannelValue::Decimal32(10.0),
        ]];

        let mod0: &dyn ProcessModbusTcpData = &m0;
        let mod1: &dyn ProcessModbusTcpData = &m1;

        let addr_out_0 = to_bit_address(ADDR_PACKED_PROCESS_OUTPUT_DATA, 0);
        let addr_in_1 = to_bit_address(ADDR_PACKED_PROCESS_INPUT_DATA, 0);

        let o0 = ModuleOffset {
            input: None,
            output: Some(addr_out_0),
        };
        let o1 = ModuleOffset {
            input: Some(addr_in_1),
            output: None,
        };

        let modules = vec![(mod0, &o0), (mod1, &o1)];

        assert!(process_output_values(&modules, &values).is_err());
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
            vec![],
        ];

        let mod0: &dyn ProcessModbusTcpData = &m0;
        let mod1: &dyn ProcessModbusTcpData = &m1;

        let addr_out_0 = to_bit_address(ADDR_PACKED_PROCESS_OUTPUT_DATA + 10, 0);
        let addr_in_1 = to_bit_address(ADDR_PACKED_PROCESS_INPUT_DATA, 0);

        let o0 = ModuleOffset {
            input: None,
            output: Some(addr_out_0),
        };
        let o1 = ModuleOffset {
            input: Some(addr_in_1),
            output: None,
        };

        let modules = vec![(mod0, &o0), (mod1, &o1)];
        assert!(process_output_values(&modules, &values).is_err());
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

        let mod0: &dyn ProcessModbusTcpData = &m0;
        let mod1: &dyn ProcessModbusTcpData = &m1;
        let mod2: &dyn ProcessModbusTcpData = &m2;

        let addr_out_0 = to_bit_address(ADDR_PACKED_PROCESS_OUTPUT_DATA + 0, 0);
        let addr_in_1 = to_bit_address(ADDR_PACKED_PROCESS_INPUT_DATA, 0);
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

        let modules = vec![(mod0, &o0), (mod1, &o1), (mod2, &o2)];
        assert!(process_output_values(&modules, &values).is_err());
    }

    #[test]
    fn test_process_output_values_with_invalid_offset_c() {
        let m0 = super::ur20_4ao_ui_16::Mod::default();
        let values = vec![vec![
            ChannelValue::Decimal32(15.0),
            ChannelValue::Decimal32(20.0),
            ChannelValue::Decimal32(20.0),
            ChannelValue::Decimal32(10.0),
        ]];
        let mod0: &dyn ProcessModbusTcpData = &m0;
        let addr_out_0 = to_bit_address(0, 0);
        let o0 = ModuleOffset {
            input: None,
            output: Some(addr_out_0),
        };
        let modules = vec![(mod0, &o0)];
        assert!(process_output_values(&modules, &values).is_err());
    }

    #[test]
    fn test_process_output_values() {
        let mut m0 = super::ur20_4ao_ui_16::Mod::default();
        let m1 = super::ur20_4ai_rtd_diag::Mod::default();
        let m2 = super::ur20_4do_p::Mod::default();
        let m3 = super::ur20_4do_p::Mod::default();

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

        let mod0: &dyn ProcessModbusTcpData = &m0;
        let mod1: &dyn ProcessModbusTcpData = &m1;
        let mod2: &dyn ProcessModbusTcpData = &m2;
        let mod3: &dyn ProcessModbusTcpData = &m3;

        let addr_out_0 = to_bit_address(ADDR_PACKED_PROCESS_OUTPUT_DATA, 0);
        let addr_in_1 = to_bit_address(ADDR_PACKED_PROCESS_INPUT_DATA, 0);
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

        let modules = vec![(mod0, &o0), (mod1, &o1), (mod2, &o2), (mod3, &o3)];

        let res = process_output_values(&modules, &values).unwrap();
        assert_eq!(res.len(), 5);
        assert_eq!(res[0], 0x0); // channel is disabled
        assert_eq!(res[1], 0x6C00);
        assert_eq!(res[2], 0x0); // channel is disabled
        assert_eq!(res[3], 0x3600);
        assert_eq!(res[4], 0b_0000_1100_0000_0010);
    }

    #[test]
    fn test_param_addresses_and_register_counts() {
        assert_eq!(param_addresses_and_register_counts(&[]), vec![]);
        assert_eq!(
            param_addresses_and_register_counts(&[ModuleType::UR20_4DI_P]),
            vec![(0xC000, 4)]
        );
        assert_eq!(
            param_addresses_and_register_counts(&[
                ModuleType::UR20_4DI_P,
                ModuleType::UR20_4DO_P,
                ModuleType::UR20_4AI_RTD_DIAG,
            ]),
            vec![(0xC000, 4), (0xC100, 4), (0xC200, 29)]
        );
    }

    #[test]
    fn validate_coupler_config_data() {
        assert!(CouplerConfig {
            modules: vec![],
            offsets: vec![],
            params: vec![],
        }
        .validate()
        .is_ok());
        assert!(CouplerConfig {
            modules: vec![ModuleType::UR20_4DI_P],
            offsets: vec![0xFFFF, 0x0000],
            params: vec![vec![0; 4]],
        }
        .validate()
        .is_ok());
        assert!(CouplerConfig {
            modules: vec![ModuleType::UR20_4DI_P],
            offsets: vec![0xFFFF, 0x0000],
            params: vec![],
        }
        .validate()
        .is_err());
        assert!(CouplerConfig {
            modules: vec![ModuleType::UR20_4DI_P],
            offsets: vec![],
            params: vec![vec![0; 4]],
        }
        .validate()
        .is_err());
        assert!(CouplerConfig {
            modules: vec![ModuleType::UR20_4DI_P],
            offsets: vec![0xFFFF],
            params: vec![],
        }
        .validate()
        .is_err());
    }

    #[test]
    fn create_new_coupler_instance() {
        let cfg = CouplerConfig {
            modules: vec![ModuleType::UR20_4DI_P, ModuleType::UR20_1COM_232_485_422],
            offsets: vec![0xFFFF, 0x0000, 0x8000, 0x0008],
            params: vec![vec![0; 4], vec![0; 10]],
        };

        let mut invalid_cfg = cfg.clone();
        invalid_cfg.params = vec![];
        let c = Coupler::new(&cfg).unwrap();

        assert!(Coupler::new(&invalid_cfg).is_err());
        assert_eq!(c.modules.len(), 2);
        assert_eq!(c.processors.len(), 1);
        assert_eq!(c.offsets.len(), 2);
        assert_eq!(c.in_values.len(), 0);
        assert_eq!(c.out_values.len(), 0);
        assert_eq!(c.write.len(), 0);
    }

    #[test]
    fn process_in_out_data_with_coupler() {
        use crate::ur20_1com_232_485_422::*;
        use num_traits::ToPrimitive;

        let cfg = CouplerConfig {
            modules: vec![
                ModuleType::UR20_4DI_P,
                ModuleType::UR20_4DO_P,
                ModuleType::UR20_1COM_232_485_422,
            ],
            offsets: vec![
                0xFFFF,
                0x0000,
                0x8000,
                0xFFFF,
                to_bit_address(0x0801, 0),
                to_bit_address(0x0001, 0),
            ],
            params: vec![
                vec![0; 4],
                vec![0; 4],
                #[cfg_attr(rustfmt, rustfmt_skip)]
                vec![
                    ProcessDataLength::EightBytes.to_u16().unwrap(),
                    OperatingMode::RS232.to_u16().unwrap(),
                    0, 0, 0, 0, 0, 0, 0, 0,
                ],
            ],
        };
        let mut c = Coupler::new(&cfg).unwrap();
        let process_input_data = vec![
            0b_0101,               // module input for DI_P
            0b_00000100_1111_0001, // len & status
            0,                     // data
            0xABCD,                // data
            0,
        ];
        let process_output_data = vec![0b_11_00, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        // make sure the initialization process evolves
        let process_output_data = c.next(&process_input_data, &process_output_data).unwrap();
        let process_output_data = c.next(&process_input_data, &process_output_data).unwrap();
        let process_output_data = c.next(&process_input_data, &process_output_data).unwrap();

        {
            let inputs = c.inputs();
            let outputs = c.outputs();

            assert_eq!(inputs.len(), 3);
            assert_eq!(outputs.len(), 3);

            assert_eq!(inputs[0].len(), 4);
            assert_eq!(outputs[0].len(), 4);

            assert_eq!(inputs[0][0], ChannelValue::Bit(true));
            assert_eq!(inputs[0][1], ChannelValue::Bit(false));
            assert_eq!(inputs[0][2], ChannelValue::Bit(true));
            assert_eq!(inputs[0][3], ChannelValue::Bit(false));

            assert_eq!(outputs[1][0], ChannelValue::Bit(false));
            assert_eq!(outputs[1][1], ChannelValue::Bit(false));
            assert_eq!(outputs[1][2], ChannelValue::Bit(true));
            assert_eq!(outputs[1][3], ChannelValue::Bit(true));

            assert_eq!(outputs[0], vec![ChannelValue::None; 4]);
            assert_eq!(inputs[1], vec![ChannelValue::None; 4]);

            assert_eq!(inputs[2], vec![ChannelValue::Bytes(vec![0, 0, 0xCD, 0xAB])]);
            assert_eq!(outputs[2], vec![ChannelValue::None]);
        }

        c.set_output(
            &Address {
                module: 2,
                channel: 0,
            },
            ChannelValue::Bytes(b"Hello modbus coupler!".to_vec()),
        )
        .unwrap();
        c.set_output(
            &Address {
                module: 1,
                channel: 1,
            },
            ChannelValue::Bit(true),
        )
        .unwrap();
        assert!(c
            .set_output(
                &Address {
                    module: 3,
                    channel: 0,
                },
                ChannelValue::Bit(true)
            )
            .is_err());
        assert!(c
            .set_output(
                &Address {
                    module: 2,
                    channel: 1,
                },
                ChannelValue::Bit(true)
            )
            .is_err());

        assert_eq!(c.write.len(), 2);

        let process_input_data = vec![0b_0101, 0, 0, 0, 0];
        let process_output_data = c.next(&process_input_data, &process_output_data).unwrap();
        assert_eq!(c.write.len(), 0);
        {
            let inputs = c.inputs();
            let outputs = c.outputs();
            assert_eq!(outputs[1][1], ChannelValue::Bit(false));
            assert_eq!(inputs[2][0], ChannelValue::None);
            assert_eq!(outputs[2][0], ChannelValue::None);
        }
        let process_output_data = c.next(&process_input_data, &process_output_data).unwrap();
        {
            let inputs = c.inputs();
            let outputs = c.outputs();
            assert_eq!(outputs[1][1], ChannelValue::Bit(true));
            assert_eq!(inputs[2][0], ChannelValue::None);
            assert_eq!(outputs[2][0], ChannelValue::Bytes(b"Hello ".to_vec()));
        }
        let process_output_data = c.next(&process_input_data, &process_output_data).unwrap();
        {
            let outputs = c.outputs();
            assert_eq!(outputs[2][0], ChannelValue::None);
        }

        let process_input_data = vec![
            0b_0101,               // module input for DI_P
            0b_00000101_1111_1001, // len & status (bit 3&4: RX_CNT , bit 5&6: TX_CNT_ACK)
            0xDDEE,                // data
            0xFFFF,                // data
            0x00AA,                // data
        ];
        let _process_output_data = c.next(&process_input_data, &process_output_data).unwrap();

        assert!(c.reader(0).is_none());
        assert!(c.reader(1).is_none());
        assert!(c.reader(2).is_some());
        assert!(c.writer(0).is_none());
        assert!(c.writer(1).is_none());
        assert!(c.writer(2).is_some());
        let mut buf = [0; 20];
        let reader = c.reader(2).unwrap();
        let len_0 = reader.read(&mut buf).unwrap();
        let len_1 = reader.read(&mut buf).unwrap();
        assert_eq!(len_0, 9);
        assert_eq!(len_1, 0);
        assert_eq!(
            &buf[0..9],
            &[0, 0, 0xCD, 0xAB, 0xEE, 0xDD, 0xFF, 0xFF, 0xAA]
        );
    }

    #[test]
    fn test_module_list_from_registers() {
        assert_eq!(
            module_list_from_registers(&vec![]).err().unwrap(),
            Error::RegisterCount
        );
        assert_eq!(
            module_list_from_registers(&vec![0xAB0C]).err().unwrap(),
            Error::RegisterCount
        );
        assert_eq!(
            module_list_from_registers(&vec![0x0101, 0x2FA0]).unwrap(),
            vec![ModuleType::UR20_4DO_P]
        );
    }
}
