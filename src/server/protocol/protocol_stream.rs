use std::net::SocketAddr;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use super::{message::Message, Header};

#[derive(Debug)]
pub struct ProtocolStream {
    stream: TcpStream,
}

impl ProtocolStream {
    pub fn wrap_tcp(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub fn peer_addr(&self) -> Result<SocketAddr, std::io::Error> {
        self.stream.peer_addr()
    }

    pub async fn read_message(&mut self) -> Result<Message, super::Error> {
        let header = Header(self.stream.read_u64().await?);

        super::validate_header_version(&header)?;

        let mut payload: Vec<u8> = vec![0; header.payload_len()];

        self.stream.read_exact(&mut payload).await?;

        match Message::try_from(&payload[..]) {
            Ok(m) => Ok(m),
            Err(_) => Err(super::Error::decode_fail()),
        }
    }

    pub async fn send_message(&mut self, message: Message) -> Result<(), super::Error> {
        let payload: Box<[u8]> = message.into();

        let header = super::generate_header(payload.len())?;

        self.stream.write_u64(header.to_u64()).await?;

        self.stream.write(&payload).await?;

        Ok(())
    }
}
