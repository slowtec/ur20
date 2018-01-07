//! Modbus TCP fieldbus coupler UR20-FBC-MOD-TCP

pub type Word = u16;
pub type RegisterAddress = u16;
pub type BitAddress = u16;
pub type BitNr = usize;

pub const ADDR_PACKED_PROCESS_INPUT_DATA  : RegisterAddress = 0x0000;
pub const ADDR_PACKED_PROCESS_OUTPUT_DATA : RegisterAddress = 0x0800;
pub const ADDR_PROCESS_OUTPUT_LEN         : RegisterAddress = 0x1010;
pub const ADDR_PROCESS_INPUT_LEN          : RegisterAddress = 0x1011;
pub const ADDR_COUPLER_ID                 : RegisterAddress = 0x1000;
pub const ADDR_COUPLER_STATUS             : RegisterAddress = 0x100C;
pub const ADDR_CURRENT_MODULE_COUNT       : RegisterAddress = 0x27FE;
pub const ADDR_CURRENT_MODULE_LIST        : RegisterAddress = 0x2A00;
pub const ADDR_MODULE_OFFSETS             : RegisterAddress = 0x2B00;

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
}
