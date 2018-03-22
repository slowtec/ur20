//! Serial communication module UR20-1COM-232-485-422

use super::*;
use util::*;

#[derive(Debug)]
pub struct Mod {
    pub mod_params: ModuleParameters,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperatingMode {
    Disabled = 0,
    RS232 = 1,
    RS485 = 2,
    RS422 = 3,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataBits {
    SevenBits = 0,
    EightBits = 1,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
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

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone, PartialEq,Eq)]
pub enum StopBit {
    OneBit  = 0,
    TwoBits = 1,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Parity {
    None = 0,
    Even = 1,
    Odd  = 2
}

#[cfg_attr(rustfmt, rustfmt_skip)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq,Eq)]
pub enum FlowControl {
    None     = 0,
    CTS_RTS  = 1,
    XON_XOFF = 2
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
        &self,
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

        let byte_count = match *process_data_length {
            ProcessDataLength::EightBytes => 8,
            ProcessDataLength::SixteenBytes => 16,
        };

        let mut msg = vec![0; byte_count];
        msg[0] = status;
        msg[1] = self.data.len() as u8;
        for (i, d) in self.data.iter().enumerate() {
            msg[2 + i] = *d;
        }
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
    fn process_input_data(&self, data: &[u16]) -> Result<Vec<ChannelValue>> {
        let buf: Vec<u8> = data.iter().fold(vec![], |mut x, elem| {
            x.push((elem & 0xff) as u8);
            x.push((elem >> 8) as u8);
            x
        });
        let current_input = ProcessInput::try_from_byte_message(&buf)?;
        Ok(vec![ChannelValue::ComRsIn(current_input)])
    }
    fn process_output_data(&self, data: &[u16]) -> Result<Vec<ChannelValue>> {
        let buf: Vec<u8> = data.iter().fold(vec![], |mut x, elem| {
            x.push((elem & 0xff) as u8);
            x.push((elem >> 8) as u8);
            x
        });
        let current_output = ProcessOutput::try_from_byte_message(&buf)?;
        Ok(vec![ChannelValue::ComRsOut(current_output)])
    }
    fn process_output_values(&self, values: &[ChannelValue]) -> Result<Vec<u16>> {
        if values.len() != 1 {
            return Err(Error::ChannelValue);
        }
        match values[0] {
            ChannelValue::ComRsOut(ref current_output) => {
                let count = self.mod_params.process_data_len.user_data_len();
                if current_output.data.len() > count {
                    return Err(Error::BufferLength);
                }
                let msg = current_output.try_into_byte_message(&self.mod_params.process_data_len)?;
                Ok(u8_to_u16(&msg))
            }
            _ => Err(Error::ChannelValue),
        }
    }
}

const CNT_MASK: u8 = 0b_0001_1000;
const CNT_ACK_MASK: u8 = 0b_0110_0000;

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

#[derive(Debug)]
pub struct MessageProcessor {
    in_data: Vec<u8>,
    out_data: Vec<Vec<u8>>,
    process_data_len: ProcessDataLength,
}

impl MessageProcessor {
    /// Create a new MessageProcessor.
    pub fn new(process_data_len: ProcessDataLength) -> MessageProcessor {
        MessageProcessor {
            in_data: vec![],
            out_data: vec![],
            process_data_len,
        }
    }

    /// Processes the current process input and output data.
    /// Returns a `ProcessOutput` object if something needs to be written.
    pub fn next(&mut self, input: &ProcessInput, output: &ProcessOutput) -> ProcessOutput {
        let mut out_msg = output.clone();
        if self.out_data.len() > 0 {
            if Self::inc_tx_cnt_ack(input.tx_cnt_ack) != output.tx_cnt {
                out_msg.tx_cnt = Self::inc_tx_cnt_ack(input.tx_cnt_ack);
                out_msg.active = true;
                out_msg.data = self.out_data.remove(0);
            }
        }
        if input.data_available {
            self.in_data.extend_from_slice(&input.data);
        }
        out_msg.rx_cnt_ack = input.rx_cnt;
        out_msg
    }

    fn inc_tx_cnt_ack(mut tx_cnt_ack: usize) -> usize {
        tx_cnt_ack += 1;
        if tx_cnt_ack > 3 {
            tx_cnt_ack = 0;
        }
        tx_cnt_ack
    }

    /// Read data form internal buffer.
    pub fn read(&mut self) -> Option<Vec<u8>> {
        if self.in_data.len() > 0 {
            Some(self.in_data.split_off(0))
        } else {
            None
        }
    }

    /// Write data to internal buffer.
    pub fn write(&mut self, data: &[u8]) {
        for c in data.chunks(self.process_data_len.user_data_len()) {
            self.out_data.push(c.to_vec());
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn try_process_input_data_data_from_empty_byte_message() {
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
    fn try_process_input_data_data_from_invalid_byte_message() {
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
    fn try_process_input_data_data_from_valid_byte_message() {
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

        assert_eq!(empty, vec![0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(flush_rx_buf[0], 0b1);
        assert_eq!(flush_tx_buf[0], 0b10);
        assert_eq!(disable_tx_hw_buffer[0], 0b100);
        assert_eq!(tx_cnt[0], 0b11000);
        assert_eq!(rx_cnt_ack[0], 0b1100000);
        assert_eq!(active[0], 0b10000000);
        assert_eq!(data, vec![0, 4, 4, 3, 2, 1, 0, 0]);
    }

    #[test]
    fn try_process_output_from_valid_byte_message() {
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
    fn test_process_input_data_with_empty_buffer() {
        let m = Mod::default();
        assert!(m.process_input_data(&vec![]).is_err());
    }

    #[test]
    fn try_process_output_data_from_valid_data() {
        let m = Mod::default();
        let data = vec![0b_0000_0011_0101_1010, 0x0E00, 7];
        let values = m.process_output_data(&data).unwrap();
        assert_eq!(values.len(), 1);
        if let ChannelValue::ComRsOut(ref out) = values[0] {
            assert_eq!(out.rx_buf_flush, false);
            assert_eq!(out.tx_buf_flush, true);
            assert_eq!(out.disable_tx_hw_buffer, false);
            assert_eq!(out.tx_cnt, 3);
            assert_eq!(out.rx_cnt_ack, 2);
            assert_eq!(out.active, false);
            assert_eq!(out.data, vec![0, 14, 7]);
        } else {
            panic!("wrong channel data");
        }
    }

    #[test]
    fn test_process_output_data_with_empty_buffer() {
        let m = Mod::default();
        assert!(m.process_output_data(&vec![]).is_err());
    }

    #[test]
    fn test_process_input_data_with_valid_input_data() {
        let m = Mod::default();
        let result = m.process_input_data(&vec![0x0600, 0, 0xABCD, 0]).unwrap();
        if let ChannelValue::ComRsIn(ref msg) = result[0] {
            assert_eq!(msg.data, vec![0, 0, 0xCD, 0xAB, 0, 0]);
        } else {
            panic!("unexpected result: {:?}", result);
        }
    }

    #[test]
    fn test_process_output_values_with_invalid_input_len() {
        let m = Mod::default();
        assert!(m.process_output_values(&vec![]).is_err());
        assert!(m.process_output_values(&vec![
            ChannelValue::ComRsIn(ProcessInput::default()),
            ChannelValue::ComRsIn(ProcessInput::default()),
        ]).is_err());
    }

    #[test]
    fn test_process_output_values_with_invalid_channel_data() {
        let m = Mod::default();
        assert!(
            m.process_output_values(&vec![ChannelValue::Decimal32(0.0)])
                .is_err()
        );
    }

    #[test]
    fn test_process_output_values_with_invalid_byte_len() {
        let mut m = Mod::default();
        let mut fourteen = ProcessOutput::default();
        fourteen.data = vec![0; 14];

        let mut fifteen = ProcessOutput::default();
        fifteen.data = vec![0; 15];

        let mut six = ProcessOutput::default();
        six.data = vec![0; 6];

        let mut seven = ProcessOutput::default();
        seven.data = vec![0; 7];

        let mut five = ProcessOutput::default();
        five.data = vec![0, 5];

        assert!(
            m.process_output_values(&vec![ChannelValue::ComRsOut(five)])
                .is_ok()
        );

        assert!(
            m.process_output_values(&vec![ChannelValue::ComRsOut(fourteen.clone())])
                .is_ok()
        );

        assert!(
            m.process_output_values(&vec![ChannelValue::ComRsOut(fifteen.clone())])
                .is_err()
        );

        m.mod_params.process_data_len = ProcessDataLength::EightBytes;
        assert!(
            m.process_output_values(&vec![ChannelValue::ComRsOut(fourteen)])
                .is_err()
        );

        assert!(
            m.process_output_values(&vec![ChannelValue::ComRsOut(seven.clone())])
                .is_err()
        );

        assert!(
            m.process_output_values(&vec![ChannelValue::ComRsOut(six.clone())])
                .is_ok()
        );
    }

    #[test]
    fn test_process_output_values() {
        let mut m = Mod::default();
        m.mod_params.process_data_len = ProcessDataLength::SixteenBytes;
        let mut out = ProcessOutput::default();
        out.data = vec![0x0A, 0x0B, 0, 0x0C];
        let res = m.process_output_values(&vec![ChannelValue::ComRsOut(out)])
            .unwrap();
        assert_eq!(res.len(), 8);
        assert_eq!(res, vec![0x0400, 0x0B0A, 0x0C00, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_eight_byte_message_processor_send_process() {
        // 1. initial state
        let mut p = MessageProcessor::new(ProcessDataLength::EightBytes);
        let mut input = ProcessInput::default();
        let mut output = ProcessOutput::default();

        // 2. first read
        input.ready = true;
        assert_eq!(input.data_available, false);

        // 3. first write
        // There is no data to send, and nothing to receive
        // so we don't need to change anything.
        assert_eq!(p.next(&input, &output), output);

        // 4. write data to processor buffer
        p.write(b"This msg is >6 bytes");

        // 5. read
        // We assume that there is still no data to receive
        // and nothing was send.
        assert_eq!(input.tx_cnt_ack, 0);
        assert_eq!(input.data_available, false);

        // 6. write
        // Now that there is data to transmit the transmission
        // counter needs to be incremented.
        output = p.next(&input, &output);
        assert_eq!(output.data, b"This m");
        assert_eq!(output.tx_cnt, 1);

        // 7. read
        // We assume the data is not fully transmitted
        // so there is no acknowledge.
        assert_eq!(input.tx_cnt_ack, 0);

        // 8. write
        // Since we have to wait for tx_cnt == tx_cnt_ack
        // the output should be unchanged.
        assert_eq!(p.next(&input, &output), output);

        // 9. read
        // We can now read the acknowledge of the first message.
        input.tx_cnt_ack = 1;

        // 10. write
        // now the next chunk can be send.
        output = p.next(&input, &output);
        assert_eq!(output.tx_cnt, 2);
        assert_eq!(output.data, b"sg is ");

        // 11: read cycle
        input.tx_cnt_ack = 2;

        // 12. write
        output = p.next(&input, &output);
        assert_eq!(output.tx_cnt, 3);
        assert_eq!(output.data, b">6 byt");

        // 13. read
        input.tx_cnt_ack = 3;

        // 14. write
        output = p.next(&input, &output);
        assert_eq!(output.tx_cnt, 0);
        assert_eq!(output.data, b"es");

        // 15: read
        input.tx_cnt_ack = 3;

        // 16: write
        assert_eq!(p.next(&input, &output), output);
    }

    #[test]
    fn test_sixteen_byte_message_processor_send_process() {
        let mut p = MessageProcessor::new(ProcessDataLength::SixteenBytes);
        let mut input = ProcessInput::default();
        let mut output = ProcessOutput::default();

        input.ready = true;
        p.write(b"This msg is >14 bytes");
        output = p.next(&input, &output);
        assert_eq!(output.data, b"This msg is >1");
        assert_eq!(output.tx_cnt, 1);
        input.tx_cnt_ack = 1;
        output = p.next(&input, &output);
        assert_eq!(output.data, b"4 bytes");
    }

    #[test]
    fn test_eight_byte_message_processor_receive_process() {
        let mut p = MessageProcessor::new(ProcessDataLength::EightBytes);
        let mut input = ProcessInput::default();
        let mut output = ProcessOutput::default();

        input.ready = true;
        assert_eq!(input.data_available, false);
        output = p.next(&input, &output);
        assert_eq!(p.read(), None);

        input.data = b"a msg".to_vec();
        input.data_available = true;
        output = p.next(&input, &output);
        assert_eq!(p.read(), Some(b"a msg".to_vec()));
        assert_eq!(p.read(), None);

        input.data = b"Foo".to_vec();
        output = p.next(&input, &output);
        input.data = b" bar".to_vec();
        output = p.next(&input, &output);
        input.data = b" baz".to_vec();
        p.next(&input, &output);
        assert_eq!(p.read(), Some(b"Foo bar baz".to_vec()));
    }

    #[test]
    fn test_message_processor_send_process_with_outdated_tx_cnt() {
        let test = |ack, cnt, cnt_next, data| {
            let mut p = MessageProcessor::new(ProcessDataLength::EightBytes);
            let mut input = ProcessInput::default();
            let mut output = ProcessOutput::default();
            input.ready = true;
            p.out_data = vec![b"some data".to_vec()];
            input.tx_cnt_ack = ack;
            output.tx_cnt = cnt;
            output = p.next(&input, &output);
            assert_eq!(output.tx_cnt, cnt_next);
            assert_eq!(output.data.len() > 0, data);
        };

        test(0, 0, 1, true);
        test(0, 1, 1, false);
        test(0, 2, 1, true);
        test(0, 3, 1, true);

        test(1, 0, 2, true);
        test(1, 1, 2, true);
        test(1, 2, 2, false);
        test(1, 3, 2, true);

        test(2, 0, 3, true);
        test(2, 1, 3, true);
        test(2, 2, 3, true);
        test(2, 3, 3, false);

        test(3, 0, 0, false);
        test(3, 1, 0, true);
        test(3, 2, 0, true);
        test(3, 3, 0, true);
    }

    #[test]
    fn test_inc_tx_cnt_ack() {
        assert_eq!(MessageProcessor::inc_tx_cnt_ack(0), 1);
        assert_eq!(MessageProcessor::inc_tx_cnt_ack(1), 2);
        assert_eq!(MessageProcessor::inc_tx_cnt_ack(2), 3);
        assert_eq!(MessageProcessor::inc_tx_cnt_ack(3), 0);
        assert_eq!(MessageProcessor::inc_tx_cnt_ack(4), 0);
    }
}
