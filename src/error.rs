use std::{fmt, io};

/// UR20 specific errors.
#[derive(Debug, PartialEq)]
pub enum Error {
    UnknownModule,
    UnknownCategory,
    BufferLength,
    SequenceNumber,
    DataLength,
    RegisterCount,
    ChannelParameter,
    ChannelValue,
    ModuleOffset,
    Address,
    Io(String), // TODO
}

#[rustfmt::skip]
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
            Error::ChannelValue     => write!(f, "invalid channel value(s)"),
            Error::ModuleOffset     => write!(f, "invalid module offset"),
            Error::Address          => write!(f, "invalid module address"),
            Error::Io(ref err)      => write!(f, "I/O error: {}", err),
        }
    }
}

#[rustfmt::skip]
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
            Error::ChannelValue     => "invalid channel value(s)",
            Error::ModuleOffset     => "invalid module offset",
            Error::Address          => "invalid module address",
            Error::Io(ref err)      => err
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(format!("{}", e))
    }
}
