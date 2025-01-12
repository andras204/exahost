use std::net::SocketAddr;

use flume::{Receiver, Sender};
use log::{debug, error, info};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    select,
};

mod message;

use message::{Header, Message, NetworkFrame};

use super::link_actions::{LinkInput, LinkOutput};

pub struct LinkHandle {
    peer_addr: SocketAddr,
    outgoing_send: Sender<LinkInput>,
    incoming_recv: Receiver<LinkOutput>,
}

impl LinkHandle {}

pub struct Link {
    stream: TcpStream,
    incoming_send: Sender<LinkOutput>,
    outgoing_recv: Receiver<LinkInput>,
}

impl Link {
    pub async fn handle_connection(&mut self) {
        let recv = self.outgoing_recv.clone();
        loop {
            select! {
                res = recv.recv_async() => match res {
                    Ok(msg) => match self.send_message(Message::Yes).await {
                        Ok(_) => (),
                        Err(_) => unimplemented!(),
                    },
                    Err(_) => {
                        info!(
                            "[net] closing connection to {}",
                            self.stream.peer_addr().unwrap()
                        );
                        return;
                    },
                },
                res = self.recv_message() => match res {
                    Ok(_) => (),
                    Err(_) => {
                        info!(
                            "[net] connection to {} closed by peer",
                            self.stream.peer_addr().unwrap()
                        );
                        return;
                    },
                },
            }
        }
    }

    async fn recv_message(&mut self) -> Result<Message, ()> {
        let version = match self.stream.read_u32().await {
            Ok(v) => v,
            Err(e) => {
                error!("[net] cannot read message header: {}", e);
                return Err(());
            }
        };

        let len = match self.stream.read_u32().await {
            Ok(l) => l,
            Err(e) => {
                error!("[net] cannot read message header: {}", e);
                return Err(());
            }
        };

        debug!("[net] recieved header: ver: {}, len: {}", version, len);

        let mut payload_bytes: Vec<u8> = vec![0; len as usize];

        match self.stream.read_exact(&mut payload_bytes).await {
            Ok(_) => (),
            Err(e) => {
                error!("[net] cannot read message payload: {}", e);
                return Err(());
            }
        };

        if version != Header::default_message_version() {
            error!(
                "[net] message version mismatch: self: {}, peer: {}",
                Header::default_message_version(),
                version
            );
            return Err(());
        }

        match Message::try_from(&payload_bytes[..]) {
            Ok(msg) => Ok(msg),
            Err(e) => {
                error!("[net] cannot decode message payload: {}", e);
                return Err(());
            }
        }
    }

    async fn send_message(&mut self, msg: Message) -> Result<(), std::io::Error> {
        let nf = NetworkFrame::from(&msg);
        self.stream.write_u32(nf.header.version).await?;
        self.stream.write_u32(nf.header.payload_len).await?;
        self.stream.write(&nf.payload).await?;
        Ok(())
    }
}

pub fn wrap_tcp(connection: TcpStream) -> (Link, LinkHandle) {
    let (s_in, r_in) = flume::unbounded();
    let (s_out, r_out) = flume::unbounded();
    let addr = connection.peer_addr().unwrap();
    (
        Link {
            stream: connection,
            incoming_send: s_in,
            outgoing_recv: r_out,
        },
        LinkHandle {
            peer_addr: addr,
            outgoing_send: s_out,
            incoming_recv: r_in,
        },
    )
}
