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

    pub async fn read_message(&mut self) -> Result<Message, super::Error> {
        let header = ProtocolHeader::from_u64(self.stream.read_u64().await?);

        if !super::is_header_version_valid(&header) {
            return Err(super::Error::version_mismatch());
        };

        let mut payload: Vec<u8> = vec![0; header.payload_len()];

        self.stream.read_exact(&mut payload).await?;

        match Message::try_from(&payload[..]) {
            Ok(m) => Ok(m),
            Err(_) => Err(super::Error::decode_fail()),
        }
    }

    pub async fn send_message(&mut self, message: Message) -> Result<(), super::Error> {
        let payload: Box<[u8]> = message.into();

        let header = match super::generate_header(payload.len()) {
            Ok(h) => h,
            Err(_) => {
                return Err(super::Error::too_long());
            }
        };

        self.stream.write_u64(header.to_u64()).await?;

        self.stream.write(&payload).await?;

        Ok(())
    }
}
