use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::exa::PackedExa;

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum Message {
    Yes,
    No,
    ConnectionRequest,
    ExaLinkRequest,
    PackedExa(PackedExa),
}

impl TryFrom<&[u8]> for Message {
    type Error = bitcode::Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        bitcode::decode(value)
    }
}

#[derive(Debug, Clone)]
pub struct NetworkFrame {
    pub header: Header,
    pub payload: Box<[u8]>,
}

impl NetworkFrame {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            header: Header::new(bytes.len() as u32),
            payload: bytes.into(),
        }
    }
}

impl From<&Message> for NetworkFrame {
    fn from(value: &Message) -> Self {
        let payload = bitcode::encode(value).into_boxed_slice();
        Self {
            header: Header::new(payload.len() as u32),
            payload,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub version: u32,
    pub payload_len: u32,
}

impl Header {
    pub fn new(len: u32) -> Self {
        let version = (env!("CARGO_PKG_VERSION_MAJOR").parse::<u32>().unwrap() << 24)
            | (env!("CARGO_PKG_VERSION_MINOR").parse::<u32>().unwrap() << 16)
            | (env!("CARGO_PKG_VERSION_PATCH").parse::<u32>().unwrap() << 8);
        Self {
            version,
            payload_len: len,
        }
    }

    pub fn default_message_version() -> u32 {
        (env!("CARGO_PKG_VERSION_MAJOR").parse::<u32>().unwrap() << 24)
            | (env!("CARGO_PKG_VERSION_MINOR").parse::<u32>().unwrap() << 16)
            | (env!("CARGO_PKG_VERSION_PATCH").parse::<u32>().unwrap() << 8)
    }
}
