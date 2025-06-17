use std::net::SocketAddr;

use crate::cli::command::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerCommand {
    Connect {
        addr: SocketAddr,
        link_id: Option<i16>,
    },
    Disconnect {
        link_id: i16,
    },
    DisconnectAll,
    NotifyExaLink,
}

impl TryFrom<Command> for ServerCommand {
    type Error = ();
    fn try_from(value: Command) -> Result<Self, Self::Error> {
        match value {
            Command::Connect { addr, link_id } => Ok(Self::Connect { addr, link_id }),
            Command::Disconnect { link_id } => Ok(Self::Disconnect { link_id }),
            Command::DisconnectAll => Ok(Self::DisconnectAll),
            _ => Err(()),
        }
    }
}
