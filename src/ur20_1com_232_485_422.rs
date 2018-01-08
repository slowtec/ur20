//! Serial communication module UR20-1COM-232-485-422

use super::*;
use util::*;

#[derive(Debug)]
pub struct Mod {
    pub mod_params: ModuleParameters,
    pub current_output: ProcessOutput,
    pub current_input: ProcessInput,
}

#[derive(Debug, Clone)]
pub struct ProcessInput {
    /// Indicates if there is a telegramm in the receive buffer or not.
    pub data_available: bool,
    /// If this flag is set there are only 10 characters left in the receive
    /// buffer.
    pub buffer_nearly_full: bool,
    /// The receiving sequence number.
    /// The sequence is: 0,1,2,3,0,...
    pub rx_cnt: usize,
    /// Acknowledges that the transmitted data of the corresponding sequence has
    /// been taken over successfully.
    /// The value is a copy of `tx_cnt` of the process output data.
    pub tx_cnt_ack: usize,
    /// Indicates whether the communication with the device is without fault or
    /// not.
    pub ready: bool,
    /// User data of the transfered telegramm segment
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ProcessOutput {
    /// This flag controls whether the receive buffer will be cleared
    /// or not.
    pub rx_buf_flush: bool,
    /// This flag controls whether the transmit buffer will be cleared
    /// or not.
    pub tx_buf_flush: bool,
    /// This flag controls the hardware transmit buffer:
    ///
    /// - `false`:  The hardware transmit buffer is released.
    ///             A character will be sent as soon as it reaches the buffer.
    /// - `true`:   The hardware transmit buffer is locked.
    ///             Characters will only be sent, when the flag is set to
    ///             `false` again.
    pub disable_tx_hw_buffer: bool,
    /// The transmitting sequence number.
    /// The sequence is: 0,1,2,3,0,...
    pub tx_cnt: usize,
    /// Acknowledges that the received data of the corresponding sequence has
    /// been taken over successfully.
    /// The sequence is: 0,1,2,3,0,...
    pub rx_cnt_ack: usize,
    /// The communication status.
    pub active: bool,
    /// User data of the transfered telegramm segment
    pub data: Vec<u8>,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone)]
pub struct ModuleParameters {
    pub operating_mode: OperatingMode,
    pub data_bits: DataBits,
    pub baud_rate: BaudRate,
    pub stop_bit: StopBit,
    pub parity: Parity,
    pub flow_control: FlowControl,
    pub XON_char: char,
    pub XOFF_char: char,
    pub terminating_resistor: bool,
    pub process_data_len: ProcessDataLength,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum OperatingMode {
    Disabled = 0,
    RS232 = 1,
    RS485 = 2,
    RS422 = 3,
}

#[derive(Debug, Clone)]
pub enum DataBits {
    SevenBits = 0,
    EightBits = 1,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum BaudRate {
    Baud_300    = 0,
    Baud_600    = 1,
    Baud_1200   = 2,
    Baud_2400   = 3,
    Baud_4800   = 4,
    Baud_9600   = 5,
    Baud_14400  = 6,
    Baud_19200  = 7,
    Baud_28800  = 8,
    Baud_38400  = 9,
    Baud_57600  = 10,
    Baud_115200 = 11,
}

#[derive(Debug, Clone)]
pub enum StopBit {
    OneBit  = 0,
    TwoBits = 1,
}

#[derive(Debug, Clone)]
pub enum Parity {
    None = 0,
    Even = 1,
    Odd  = 2
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum FlowControl {
    None     = 0,
    CTS_RTS  = 1,
    XON_XOFF = 2
}

#[derive(Debug, Clone)]
pub enum ProcessDataLength {
    EightBytes,
    SixteenBytes,
}

impl ProcessInput {
    pub fn try_from_byte_message(bytes: &[u8]) -> Result<Self> {

        if bytes.len() < 2 {
            return Err(Error::BufferLength);
        }

        let status = bytes[0];
        let data_len = bytes[1] as usize;

        if bytes.len() < data_len + 2 {
            return Err(Error::BufferLength);
        }

        let msg = ProcessInput {
            data_available: test_bit(status, 0),
            buffer_nearly_full: test_bit(status, 1),
            rx_cnt: cnt_from_status_byte(status),
            tx_cnt_ack: cnt_ack_from_status_byte(status),
            ready: test_bit(status, 7),
            data: bytes[2..data_len + 2].into(),
        };

        Ok(msg)
    }
}

impl Default for ProcessInput {
    fn default() -> Self {
        ProcessInput {
            data_available: false,
            buffer_nearly_full: false,
            rx_cnt: 0,
            tx_cnt_ack: 0,
            ready: false,
            data: vec![],
        }
    }
}

impl Default for ProcessOutput {
    fn default() -> Self {
        ProcessOutput {
            rx_buf_flush: false,
            tx_buf_flush: false,
            disable_tx_hw_buffer: false,
            tx_cnt: 0,
            rx_cnt_ack: 0,
            active: false,
            data: vec![],
        }
    }
}

impl ProcessDataLength {
    pub fn user_data_len(&self) -> usize {
        use self::ProcessDataLength::*;
        match *self {
            EightBytes => 6,
            SixteenBytes => 14,
        }
    }
}

impl ProcessOutput {
    pub fn try_into_byte_message(
        mut self,
        process_data_length: &ProcessDataLength,
    ) -> Result<Vec<u8>> {

        if self.tx_cnt > 3 || self.rx_cnt_ack > 3 {
            return Err(Error::SequenceNumber);
        }

        if self.data.len() > process_data_length.user_data_len() {
            return Err(Error::DataLength);
        }

        let mut status = 0;

        if self.rx_buf_flush {
            status = set_bit(status, 0);
        }

        if self.tx_buf_flush {
            status = set_bit(status, 1);
        }

        if self.disable_tx_hw_buffer {
            status = set_bit(status, 2);
        }

        status = cnt_to_status_byte(self.tx_cnt, status);
        status = cnt_ack_to_status_byte(self.rx_cnt_ack, status);

        if self.active {
            status = set_bit(status, 7);
        }

        let mut msg = vec![status, self.data.len() as u8];
        msg.append(&mut self.data);
        Ok(msg)
    }

    pub fn try_from_byte_message(bytes: &[u8]) -> Result<Self> {

        if bytes.len() < 2 {
            return Err(Error::BufferLength);
        }

        let status = bytes[0];
        let data_len = bytes[1] as usize;

        if bytes.len() < data_len + 2 {
            return Err(Error::BufferLength);
        }

        let msg = ProcessOutput {
            rx_buf_flush: test_bit(status, 0),
            tx_buf_flush: test_bit(status, 1),
            disable_tx_hw_buffer: test_bit(status, 2),
            tx_cnt: cnt_from_status_byte(status),
            rx_cnt_ack: cnt_ack_from_status_byte(status),
            active: test_bit(status, 7),
            data: bytes[2..data_len + 2].into(),
        };

        Ok(msg)

    }
}

impl Default for ModuleParameters {
    fn default() -> Self {
        ModuleParameters {
            operating_mode: OperatingMode::Disabled,
            data_bits: DataBits::EightBits,
            baud_rate: BaudRate::Baud_9600,
            stop_bit: StopBit::OneBit,
            parity: Parity::None,
            flow_control: FlowControl::None,
            XON_char: 17 as char,
            XOFF_char: 19 as char,
            terminating_resistor: false,
            process_data_len: ProcessDataLength::SixteenBytes,
        }
    }
}

impl Default for Mod {
    fn default() -> Self {
        Mod {
            mod_params: ModuleParameters::default(),
            current_input: ProcessInput::default(),
            current_output: ProcessOutput::default(),
        }
    }
}

impl Module for Mod {
    fn process_input_byte_count(&self) -> usize {
        match self.mod_params.process_data_len {
            ProcessDataLength::EightBytes => 8,
            ProcessDataLength::SixteenBytes => 16,
        }
    }
    fn process_output_byte_count(&self) -> usize {
        match self.mod_params.process_data_len {
            ProcessDataLength::EightBytes => 8,
            ProcessDataLength::SixteenBytes => 16,
        }
    }
    fn module_type(&self) -> ModuleType {
        ModuleType::UR20_1COM_232_485_422
    }
    fn process_input(&mut self, data: &[u16]) -> Result<Vec<ChannelValue>> {
        let buf: Vec<u8> = data.iter().fold(vec![], |mut x, elem| {
            x.push((elem & 0xff) as u8);
            x.push((elem >> 8) as u8);
            x
        });
        self.current_input = ProcessInput::try_from_byte_message(&buf)?;
        Ok(vec![ChannelValue::Bytes(self.current_input.data.clone())])
    }
    fn values_into_output_data(&mut self, values: &[ChannelValue]) -> Result<Vec<u16>> {
        if values.len() != 1 {
            return Err(Error::ChannelValue);
        }
        match values[0] {
            ChannelValue::Bytes(ref bytes) => {
                let count = self.mod_params.process_data_len.user_data_len();
                if bytes.len() > count {
                    return Err(Error::BufferLength);
                }
                self.current_output.data = bytes.clone();
                let msg = self.current_output.clone().try_into_byte_message(
                    &self.mod_params.process_data_len,
                )?;
                Ok(u8_to_u16(&msg))
            }
            _ => {
                return Err(Error::ChannelValue);
            }
        }
    }
}

const CNT_MASK     : u8 = 0b00011000;
const CNT_ACK_MASK : u8 = 0b01100000;

fn cnt_from_status_byte(byte: u8) -> usize {
    ((CNT_MASK & byte) >> 3) as usize
}

fn cnt_to_status_byte(cnt: usize, mut byte: u8) -> u8 {
    byte |= CNT_MASK & ((cnt as u8) << 3);
    byte
}

fn cnt_ack_from_status_byte(byte: u8) -> usize {
    ((CNT_ACK_MASK & byte) >> 5) as usize
}

fn cnt_ack_to_status_byte(cnt: usize, mut byte: u8) -> u8 {
    byte |= CNT_ACK_MASK & ((cnt as u8) << 5);
    byte
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn try_process_input_data_from_empty_byte_message() {
        let byte_msg = vec![0, 0];
        let msg = ProcessInput::try_from_byte_message(&byte_msg).unwrap();
        assert_eq!(msg.data_available, false);
        assert_eq!(msg.buffer_nearly_full, false);
        assert_eq!(msg.rx_cnt, 0);
        assert_eq!(msg.tx_cnt_ack, 0);
        assert_eq!(msg.ready, false);
        assert_eq!(msg.data, vec![]);
    }

    #[test]
    fn try_process_input_data_from_invalid_byte_message() {
        let too_small_err = ProcessInput::try_from_byte_message(&vec![0]).err().unwrap();
        let missmatched_len_err = ProcessInput::try_from_byte_message(&vec![0, 5, 0])
            .err()
            .unwrap();
        let ok_res = ProcessInput::try_from_byte_message(&vec![0, 5, 0, 0, 0, 0, 0]);
        assert_eq!(too_small_err, Error::BufferLength);
        assert_eq!(missmatched_len_err, Error::BufferLength);
        assert!(ok_res.is_ok());
    }

    #[test]
    fn try_process_input_data_from_valid_byte_message() {
        let byte_msg = vec![0b11110001, 3, 0x0, 0xf, 0x5];
        let msg = ProcessInput::try_from_byte_message(&byte_msg).unwrap();
        assert_eq!(msg.data_available, true);
        assert_eq!(msg.buffer_nearly_full, false);
        assert_eq!(msg.rx_cnt, 2);
        assert_eq!(msg.tx_cnt_ack, 3);
        assert_eq!(msg.ready, true);
        assert_eq!(msg.data, vec![0, 15, 5]);
    }

    #[test]
    fn try_invalid_process_output_data_into_byte_message() {
        let mut msg1 = ProcessOutput::default();
        let mut msg2 = ProcessOutput::default();
        let mut msg3 = ProcessOutput::default();
        msg1.tx_cnt = 4;
        msg2.rx_cnt_ack = 4;
        msg3.data = vec![0, 0, 0, 0, 0, 0, 0];
        let err1 = msg1.try_into_byte_message(&ProcessDataLength::EightBytes)
            .err()
            .unwrap();
        let err2 = msg2.try_into_byte_message(&ProcessDataLength::EightBytes)
            .err()
            .unwrap();
        let err3 = msg3.try_into_byte_message(&ProcessDataLength::EightBytes)
            .err()
            .unwrap();
        assert_eq!(err1, Error::SequenceNumber);
        assert_eq!(err2, Error::SequenceNumber);
        assert_eq!(err3, Error::DataLength);
    }

    #[test]
    fn try_valid_process_output_data_into_byte_message() {

        let default = ProcessOutput::default();

        let mut msg = default.clone();
        msg.active = false;
        let empty = msg.try_into_byte_message(&ProcessDataLength::EightBytes)
            .unwrap();

        let mut msg = default.clone();
        msg.rx_buf_flush = true;
        let flush_rx_buf = msg.try_into_byte_message(&ProcessDataLength::EightBytes)
            .unwrap();

        let mut msg = default.clone();
        msg.tx_buf_flush = true;
        let flush_tx_buf = msg.try_into_byte_message(&ProcessDataLength::EightBytes)
            .unwrap();

        let mut msg = default.clone();
        msg.disable_tx_hw_buffer = true;
        let disable_tx_hw_buffer = msg.try_into_byte_message(&ProcessDataLength::EightBytes)
            .unwrap();

        let mut msg = default.clone();
        msg.tx_cnt = 3;
        let tx_cnt = msg.try_into_byte_message(&ProcessDataLength::EightBytes)
            .unwrap();

        let mut msg = default.clone();
        msg.rx_cnt_ack = 3;
        let rx_cnt_ack = msg.try_into_byte_message(&ProcessDataLength::EightBytes)
            .unwrap();

        let mut msg = default.clone();
        msg.active = true;
        let active = msg.try_into_byte_message(&ProcessDataLength::EightBytes)
            .unwrap();

        let mut msg = default.clone();
        msg.data = vec![4, 3, 2, 1];
        let data = msg.try_into_byte_message(&ProcessDataLength::EightBytes)
            .unwrap();

        assert_eq!(empty, vec![0, 0]);
        assert_eq!(flush_rx_buf, vec![0b1, 0]);
        assert_eq!(flush_tx_buf, vec![0b10, 0]);
        assert_eq!(disable_tx_hw_buffer, vec![0b100, 0]);
        assert_eq!(tx_cnt, vec![0b11000, 0]);
        assert_eq!(rx_cnt_ack, vec![0b1100000, 0]);
        assert_eq!(active, vec![0b10000000, 0]);
        assert_eq!(data, vec![0, 4, 4, 3, 2, 1]);
    }

    #[test]
    fn try_process_output_data_from_valid_byte_message() {
        let byte_msg = vec![0b01011010, 3, 0x0, 0xe, 0x7];
        let msg = ProcessOutput::try_from_byte_message(&byte_msg).unwrap();
        assert_eq!(msg.rx_buf_flush, false);
        assert_eq!(msg.tx_buf_flush, true);
        assert_eq!(msg.disable_tx_hw_buffer, false);
        assert_eq!(msg.tx_cnt, 3);
        assert_eq!(msg.rx_cnt_ack, 2);
        assert_eq!(msg.active, false);
        assert_eq!(msg.data, vec![0, 14, 7]);
    }

    #[test]
    fn test_process_input_with_empty_buffer() {
        let mut m = Mod::default();
        assert!(m.process_input(&vec![]).is_err());
    }

    #[test]
    fn test_process_input_with_valid_input_data() {
        let mut m = Mod::default();
        assert!(m.current_input.data.is_empty());
        assert!(m.process_input(&vec![0x0600, 0, 0xABCD, 0]).is_ok());
        assert_eq!(m.current_input.data, vec![0, 0, 0xCD, 0xAB, 0, 0]);
    }

    #[test]
    fn test_values_into_output_data_with_invalid_input_len() {
        let mut m = Mod::default();
        assert!(m.values_into_output_data(&vec![]).is_err());
        assert!(
            m.values_into_output_data(&vec![
                ChannelValue::Bytes(vec![0, 0, 0, 0, 0, 0, 0, 0]),
                ChannelValue::Bytes(vec![0, 0, 0, 0, 0, 0, 0, 0]),
            ]).is_err()
        );
    }

    #[test]
    fn test_values_into_output_data_with_invalid_channel_data() {
        let mut m = Mod::default();
        assert!(
            m.values_into_output_data(&vec![ChannelValue::Decimal32(0.0)])
                .is_err()
        );
    }

    #[test]
    fn test_values_into_output_data_with_invalid_byte_len() {

        let mut m = Mod::default();
        let fourteen = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let fifteen = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let six = vec![0, 0, 0, 0, 0, 0];
        let seven = vec![0, 0, 0, 0, 0, 0, 0];
        let five = vec![0, 0, 0, 0, 0];

        assert!(
            m.values_into_output_data(&vec![ChannelValue::Bytes(five)])
                .is_ok()
        );

        assert!(
            m.values_into_output_data(&vec![ChannelValue::Bytes(fourteen.clone())])
                .is_ok()
        );

        assert!(
            m.values_into_output_data(&vec![ChannelValue::Bytes(fifteen.clone())])
                .is_err()
        );

        m.mod_params.process_data_len = ProcessDataLength::EightBytes;
        assert!(
            m.values_into_output_data(&vec![ChannelValue::Bytes(fourteen)])
                .is_err()
        );

        assert!(
            m.values_into_output_data(&vec![ChannelValue::Bytes(seven.clone())])
                .is_err()
        );

        assert!(
            m.values_into_output_data(&vec![ChannelValue::Bytes(six.clone())])
                .is_ok()
        );
    }

    #[test]
    fn test_values_into_output_data() {
        let mut m = Mod::default();
        m.mod_params.process_data_len = ProcessDataLength::EightBytes;
        assert!(m.current_output.data.is_empty());
        let res = m.values_into_output_data(
            &vec![ChannelValue::Bytes(vec![0xA, 0xB, 0xC, 0, 0, 0xD])],
        ).unwrap();
        assert_eq!(res, vec![0x0600, 0x0B0A, 0x000C, 0x0D00]);
        assert_eq!(m.current_output.data, vec![0xA, 0xB, 0xC, 0, 0, 0xD]);
    }
}
