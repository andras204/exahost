use log::{debug, error, info};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use super::message::{Message, ProtocolHeader};

pub struct MessageStream {
    stream: TcpStream,
}

impl MessageStream {
    pub fn wrap_tcp(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub async fn read_message(&mut self) -> Result<Message, ()> {
        let header = match self.stream.read_u64().await {
            Ok(n) => ProtocolHeader::from_u64(n),
            Err(e) => {
                error!("[net] cannot read message header: {}", e);
                return Err(());
            }
        };

        if !super::is_header_version_valid(&header) {
            error!("[net] message header version mismatch");
            return Err(());
        };

        let mut payload: Vec<u8> = vec![0; header.payload_len()];

        match self.stream.read_exact(&mut payload).await {
            Ok(_) => (),
            Err(e) => {
                error!("[net] cannot read message payload: {}", e);
                return Err(());
            }
        }

        match Message::try_from(&payload[..]) {
            Ok(m) => Ok(m),
            Err(e) => {
                error!("[net] cannot parse message payload: {}", e);
                Err(())
            }
        }
    }

    pub async fn send_message(&mut self, message: Message) -> Result<(), ()> {
        let payload: Box<[u8]> = message.into();

        let header = match super::generate_header(payload.len()) {
            Ok(h) => h,
            Err(_) => {
                error!("[net] message too long to send");
                return Err(());
            }
        };

        match self.stream.write_u64(header.to_u64()).await {
            Ok(_) => (),
            Err(e) => {
                error!("[net] failed to send message header: {}", e);
                return Err(());
            }
        }

        match self.stream.write(&payload).await {
            Ok(_) => (),
            Err(e) => {
                error!("[net] failed to send message header: {}", e);
                return Err(());
            }
        }

        Ok(())
    }
}
