// use bincode;
// use flume::{Receiver, Sender};
// use serde::{Deserialize, Serialize};
// use std::{collections::HashMap, net::SocketAddr, sync::Arc, sync::Mutex, thread};

// use crate::Exa;

// use tokio::{
//     io::{self, AsyncReadExt, AsyncWriteExt, BufStream},
//     net::{TcpListener, TcpStream, ToSocketAddrs},
//     runtime,
//     select,
//     // sync::Mutex,
// };

// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// pub struct Message {
//     pub message_type: MessageType,
//     pub data: Option<Vec<u8>>,
// }

// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// pub enum MessageType {
//     ConnectionRequest,
//     ConnectionResponse,
//     ExaSendRequest,
//     ExaSendResponse,
//     ExaData,
// }

// impl Message {
//     pub fn connection_request() -> Message {
//         let data = bincode::serialize(env!("CARGO_PKG_VERSION")).unwrap();
//         Message {
//             message_type: MessageType::ConnectionRequest,
//             data: Some(data),
//         }
//     }

//     pub fn connection_response() -> Message {
//         unimplemented!()
//     }

//     pub fn send_exa_request() -> Message {
//         unimplemented!()
//     }

//     pub fn exa_data(exa: Exa) -> Message {
//         let data = bincode::serialize(&exa).unwrap();
//         Message {
//             message_type: MessageType::ExaData,
//             data: Some(data),
//         }
//     }
// }

// #[derive(Debug, Clone, Copy)]
// enum ConnectionError {
//     Closed,
//     BadMessage,
// }

// struct Link {
//     input_recv: Receiver<Message>,
//     output_send: Sender<Message>,
//     stream: BufStream<TcpStream>,
//     peer_addr: SocketAddr,
// }

// impl Link {
//     pub fn new(recv: Receiver<Message>, send: Sender<Message>, stream: TcpStream) -> Self {
//         Self {
//             input_recv: recv,
//             output_send: send,
//             peer_addr: stream.peer_addr().unwrap(),
//             stream: BufStream::new(stream),
//         }
//     }

//     async fn handle_connection(&mut self) {
//         let recv = self.input_recv.clone();
//         loop {
//             select! {
//                 res = recv.recv_async() => match res {
//                     Ok(m) => match self.send_message(m).await {
//                         Ok(_) => (),
//                         Err(e) => {
//                         println!("[Error] with connection {} | {:?}",
//                             self.peer_addr,
//                             e,
//                         );
//                         return;
//                         }
//                     },
//                     Err(_) => {
//                         println!("closing connection to {}", self.peer_addr);
//                         return;
//                     },
//                 },
//                 res = self.read_message() => match res {
//                     Ok(m) => println!("{:?}", m),
//                     Err(e) => {
//                         println!("[Error] with connection {} | {:?}",
//                             self.peer_addr,
//                             e,
//                         );
//                         return;
//                     },
//                 },
//             }
//         }
//     }

//     async fn read_message(&mut self) -> Result<Message, ConnectionError> {
//         let mut header = vec![0u8; 5];
//         if self.stream.read_exact(&mut header).await.is_err() {
//             return Err(ConnectionError::Closed);
//         }

//         let msg_type = match bincode::deserialize::<MessageType>(&header[..4]) {
//             Ok(mt) => mt,
//             Err(_) => return Err(ConnectionError::BadMessage),
//         };

//         if header[4] == 0 {
//             return Ok(Message {
//                 message_type: msg_type,
//                 data: None,
//             });
//         }

//         let mut len_buf = vec![0u8; 8];
//         if self.stream.read_exact(&mut len_buf).await.is_err() {
//             return Err(ConnectionError::Closed);
//         }

//         let len = match bincode::deserialize::<usize>(&len_buf) {
//             Ok(len) => len,
//             Err(_) => return Err(ConnectionError::BadMessage),
//         };

//         let mut data = vec![0u8; len];
//         if self.stream.read_exact(&mut data).await.is_err() {
//             return Err(ConnectionError::Closed);
//         }

//         header.append(&mut len_buf);
//         header.append(&mut data);
//         match bincode::deserialize::<Message>(&header) {
//             Ok(m) => Ok(m),
//             Err(_) => Err(ConnectionError::BadMessage),
//         }
//     }

//     async fn send_message(&mut self, msg: Message) -> Result<(), ConnectionError> {
//         let bin = bincode::serialize(&msg).unwrap();
//         if self.stream.write_all(&bin).await.is_err() {
//             return Err(ConnectionError::Closed);
//         }
//         if self.stream.flush().await.is_err() {
//             return Err(ConnectionError::Closed);
//         }
//         Ok(())
//     }
// }

// #[derive(Debug)]
// struct LinkHandle {
//     sender: Sender<Message>,
//     reciever: Receiver<Message>,
// }

// impl LinkHandle {
//     fn new(sender: Sender<Message>, reciever: Receiver<Message>) -> Self {
//         Self { sender, reciever }
//     }

//     fn send(&self, message: Message) -> Result<(), ()> {
//         match self.sender.send(message) {
//             Ok(_) => Ok(()),
//             Err(_) => Err(()),
//         }
//     }

//     async fn send_async(&self, message: Message) -> Result<(), ()> {
//         match self.sender.send_async(message).await {
//             Ok(_) => Ok(()),
//             Err(_) => Err(()),
//         }
//     }

//     fn recv(&self) -> Result<Message, ()> {
//         match self.reciever.recv() {
//             Ok(m) => Ok(m),
//             Err(_) => Err(()),
//         }
//     }

//     async fn recv_async(&self) -> Result<Message, ()> {
//         match self.reciever.recv_async().await {
//             Ok(m) => Ok(m),
//             Err(_) => Err(()),
//         }
//     }
// }

// #[derive(Debug)]
// pub struct LinkManager {
//     link_handles: Arc<Mutex<HashMap<i16, LinkHandle>>>,
//     exa_queue: Arc<Mutex<Vec<Exa>>>,
// }

// impl Default for LinkManager {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// impl LinkManager {
//     pub fn new() -> Self {
//         Self {
//             link_handles: Arc::new(Mutex::new(HashMap::new())),
//             exa_queue: Arc::new(Mutex::new(Vec::new())),
//         }
//     }

//     pub fn send_exa(&self, link: i16, exa: Exa) -> Result<(), ()> {
//         let lhs = self.link_handles.lock().unwrap();
//         match lhs.get(&link) {
//             Some(lh) => lh.send(Message::exa_data(exa)).unwrap(),
//             None => return Err(()),
//         }
//         Ok(())
//     }

//     pub fn recieve_exas(&self) -> Option<Vec<Exa>> {
//         let mut exa_q = self.exa_queue.lock().unwrap();
//         if exa_q.len() == 0 {
//             return None;
//         }
//         Some(exa_q.drain(..).collect())
//     }

//     pub fn start_listening(&self, addr: impl ToSocketAddrs + Send + 'static) {
//         let link_handles = self.link_handles.clone();
//         thread::spawn(move || {
//             let rt = runtime::Builder::new_current_thread()
//                 .enable_all()
//                 .build()
//                 .unwrap();
//             rt.block_on(async move {
//                 Self::listen_loop(addr, link_handles).await.unwrap();
//             })
//         });
//     }

//     async fn listen_loop(
//         addr: impl ToSocketAddrs,
//         link_handles: Arc<Mutex<HashMap<i16, LinkHandle>>>,
//     ) -> Result<(), io::Error> {
//         let listener = TcpListener::bind(addr).await?;
//         loop {
//             let (stream, peer_addr) = listener.accept().await?;
//             println!("new connection {}", peer_addr);
//             let link_handles = link_handles.clone();
//             tokio::spawn(async move {
//                 let (sender, rx) = flume::unbounded();
//                 let (tx, reciever) = flume::unbounded();
//                 let mut link = Link::new(rx, tx, stream);
//                 {
//                     // scope to release mutex sooner
//                     let mut lh = link_handles.lock().unwrap();
//                     lh.insert(1, LinkHandle::new(sender, reciever));
//                 }
//                 link.handle_connection().await;
//             });
//         }
//     }

//     async fn collect_incoming() {}
// }
