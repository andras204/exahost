use serde::{Deserialize, Serialize};
use bincode;
use flume::Receiver;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufStream},
    net::{TcpListener, TcpStream, ToSocketAddrs},
    runtime,
    select
};

use crate::Exa;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    message_type: MessageType,
    data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    ConnectionRequest,
    ConnectionResponse,
    SendExaRequest,
    SendExaResponse,
    Exa,
}

impl Message {
    pub fn connection_request() -> Message {
        let data = bincode::serialize(env!("CARGO_PKG_VERSION")).unwrap();
        Message {
            message_type: MessageType::ConnectionRequest,
            data,
        }
    }

    pub fn connection_response() -> Message {
        unimplemented!()
    }

    pub fn send_exa_request() -> Message {
        unimplemented!()
    }

    pub fn exa(exa: Exa) -> Message {
        let data = bincode::serialize(&exa).unwrap();
        Message {
            message_type: MessageType::Exa,
            data,
        }
    }

    pub fn deserialize<'a, T>(&'a self) -> T where T: Deserialize<'a> {
        bincode::deserialize::<T>(&self.data).unwrap()
    }
}

struct Link {
    recv: Receiver<Message>,
    stream: BufStream<TcpStream>,
}

impl Link {
    fn link_loop(&mut self) {
        let rt = runtime::Builder::new_current_thread().build().unwrap();
        let recv = self.recv.clone();
        rt.block_on(async {
            loop {
                select! {
                    r = recv.recv_async() => {
                        let m = r.unwrap();
                        self.send_message(m).await;
                    }
                    m = self.read_message() => {
                        println!("{:?}", m);
                    }
                }
            }
        });
    }

    async fn read_message(&mut self) -> Message {
        let mut msg_type = Vec::with_capacity(4);
        let mut msg_len = Vec::with_capacity(8);
        self.stream.read_exact(&mut msg_type).await.unwrap();
        self.stream.read_exact(&mut msg_len).await.unwrap();
        let len = bincode::deserialize::<usize>(&msg_len).unwrap();
        let mut msg_data = Vec::with_capacity(len);
        self.stream.read_exact(&mut msg_data).await.unwrap();
        let mut msg_bin = Vec::with_capacity(12 + len);
        msg_bin.append(&mut msg_type);
        msg_bin.append(&mut msg_len);
        msg_bin.append(&mut msg_data);
        bincode::deserialize(&msg_bin).unwrap()
    }

    async fn send_message(&mut self, msg: Message) {
        let bin = bincode::serialize(&msg).unwrap();
        self.stream.write_all(&bin).await.unwrap();
        self.stream.flush().await.unwrap();
    }
}

#[derive(Debug)]
pub struct LinkManager {
}

impl LinkManager {
    pub fn new() -> Self {
        Self {
        }
    }

    pub async fn start_listening(addr: impl ToSocketAddrs) {
        let listener = TcpListener::bind(addr).await.unwrap();
    }
}
