use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Error {
    UnknownModule,
    UnknownCategory,
    BufferLength,
    SequenceNumber,
    DataLength,
    RegisterCount,
    ChannelParameter,
    ModuleOffset,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::UnknownModule    => write!(f, "unknown module type"),
            Error::UnknownCategory  => write!(f, "unknown module category"),
            Error::BufferLength     => write!(f, "invalid buffer length"),
            Error::SequenceNumber   => write!(f, "invalid sequence number"),
            Error::DataLength       => write!(f, "invalid data length"),
            Error::RegisterCount    => write!(f, "invalid number of registers"),
            Error::ChannelParameter => write!(f, "invalid channel paramater(s)"),
            Error::ModuleOffset     => write!(f, "invalid module offset"),
        }
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::UnknownModule    => "unknown module type",
            Error::UnknownCategory  => "unknown module category",
            Error::BufferLength     => "invalid buffer length",
            Error::SequenceNumber   => "invalid sequence number",
            Error::DataLength       => "invalid data length",
            Error::RegisterCount    => "invalid number of registers",
            Error::ChannelParameter => "invalid channel paramater(s)",
            Error::ModuleOffset     => "invalid module offset",
        }
    }
}
