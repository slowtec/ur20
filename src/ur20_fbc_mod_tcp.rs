//! Modbus TCP fieldbus coupler UR20-FBC-MOD-TCP

pub type ModbusAddress = u16;
pub type Word = u16;
pub type BitOffset = u16;

pub const ADDR_PACKED_PROCESS_INPUT_DATA  : ModbusAddress = 0x0000;
pub const ADDR_PACKED_PROCESS_OUTPUT_DATA : ModbusAddress = 0x0800;
pub const ADDR_COUPLER_ID                 : ModbusAddress = 0x1000;
pub const ADDR_COUPLER_STATUS             : ModbusAddress = 0x100C;
pub const ADDR_CURRENT_MODULE_LIST        : ModbusAddress = 0x2A00;
pub const ADDR_MODULE_OFFSETS             : ModbusAddress = 0x2B00;

/// The packed process data offset addresses of a module.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleOffset {
    pub input: Option<BitOffset>,
    pub output: Option<BitOffset>,
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

fn word_to_offset(word: Word) -> Option<BitOffset> {
    if word == 0xFFFF {
        None
    } else {
        Some(word)
    }
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
}
