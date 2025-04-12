use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    ProtocolError(ProtocolError),
    IoError(std::io::Error),
}

impl Error {
    pub fn version_mismatch() -> Self {
        Self::ProtocolError(ProtocolError::VersionMismatch)
    }

    pub fn decode_fail() -> Self {
        Self::ProtocolError(ProtocolError::DecodeFail)
    }

    pub fn message_too_long() -> Self {
        Self::ProtocolError(ProtocolError::MessageTooLong)
    }

    pub fn invalid_message_sequence() -> Self {
        Self::ProtocolError(ProtocolError::InvalidMessageSequence)
    }
}

#[derive(Debug)]
enum ProtocolError {
    VersionMismatch,
    DecodeFail,
    MessageTooLong,
    InvalidMessageSequence,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl std::error::Error for Error {}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
