use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::exa::PackedExa;

pub struct ProtocolHeader(u64);

impl ProtocolHeader {
    pub fn version(&self) -> u32 {
        (self.0 >> 32) as u32
    }

    pub fn payload_len(&self) -> usize {
        (self.0 & (u32::MAX as u64)) as usize
    }

    pub fn from_u64(n: u64) -> Self {
        Self(n)
    }

    pub fn to_u64(self) -> u64 {
        self.0
    }
}

pub fn generate_header(payload_len: usize) -> Result<ProtocolHeader, ()> {
    if payload_len > u32::MAX as usize {
        return Err(());
    }
    Ok(ProtocolHeader(
        ((super::PROTOCOL_VERSION as u64) << 32) + (payload_len as u64),
    ))
}

pub fn is_header_version_valid(h: &ProtocolHeader) -> bool {
    h.version() == super::PROTOCOL_VERSION
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum Message {
    Request(Request),
    Response(Response),
    Action(Action),
}

impl Message {
    pub fn yes() -> Self {
        Self::Response(Response::Yes)
    }

    pub fn no() -> Self {
        Self::Response(Response::No)
    }

    pub fn connect_request(port: u16) -> Self {
        Self::Request(Request::Connect(port))
    }

    pub fn exa_request() -> Self {
        Self::Request(Request::SendExa)
    }

    pub fn exa(pexa: PackedExa) -> Self {
        Self::Action(Action::Exa(pexa))
    }

    pub fn abort() -> Self {
        Self::Action(Action::Abort)
    }

    pub fn is_yes(&self) -> bool {
        match self {
            Self::Response(r) => match r {
                Response::Yes => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_no(&self) -> bool {
        match self {
            Self::Response(r) => match r {
                Response::No => true,
                _ => false,
            },
            _ => false,
        }
    }
}

impl Into<Box<[u8]>> for Message {
    fn into(self) -> Box<[u8]> {
        bitcode::encode(&self).into_boxed_slice()
    }
}

impl TryFrom<&[u8]> for Message {
    type Error = bitcode::Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        bitcode::decode(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum Request {
    /// listening port of connection initiator
    Connect(u16),
    SendExa,
    NetMap,
    Status,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum Response {
    Yes,
    No,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum Action {
    Exa(PackedExa),
    NetMapUpdate,
    StatusUpdate,
    Abort,
}
