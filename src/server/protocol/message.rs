use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::exa::PackedExa;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum Message {
    Request(Request),
    Response(Response),
    Action(Action),
    KeepAlive,
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

    pub fn keepalive() -> Self {
        Self::KeepAlive
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
    // NetMap,
    // Status,
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
