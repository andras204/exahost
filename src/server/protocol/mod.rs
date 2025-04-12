mod error;
pub mod message;
mod protocol_stream;

pub use error::Error;
pub use protocol_stream::ProtocolStream;

pub const PROTOCOL_VERSION: u32 = 0;

pub struct Header(u64);

impl Header {
    pub fn version(&self) -> u32 {
        (self.0 >> 32) as u32
    }

    pub fn payload_len(&self) -> usize {
        (self.0 & (u32::MAX as u64)) as usize
    }

    pub fn to_u64(self) -> u64 {
        self.0
    }
}

pub fn generate_header(payload_len: usize) -> Result<Header, Error> {
    if payload_len > u32::MAX as usize {
        return Err(Error::message_too_long());
    }
    Ok(Header(
        ((PROTOCOL_VERSION as u64) << 32) + (payload_len as u64),
    ))
}

pub fn validate_header_version(h: &Header) -> Result<(), Error> {
    if h.version() == PROTOCOL_VERSION {
        Ok(())
    } else {
        Err(Error::version_mismatch())
    }
}
