use std::net::SocketAddr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    VMCommand,
    VMState,
    ServerCommand(ServerCommand),
    ServerState,
}

impl Message {
    pub fn server_connect(addr: SocketAddr, link_id: Option<i16>) -> Self {
        Self::ServerCommand(ServerCommand::Connect(addr, link_id))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerCommand {
    Connect(SocketAddr, Option<i16>),
    NotifyExaLink,
}
