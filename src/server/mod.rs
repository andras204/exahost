use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    thread::{self, JoinHandle},
};

use flume::{Receiver, Sender};
use log::info;
use tokio::{
    net::TcpListener,
    runtime::{self},
    sync::Mutex,
};

use crate::{exa::PackedExa, vm::bridge::VMBridge};

pub mod link;
pub mod link_actions;
mod protocol;

#[derive(Debug)]
pub struct Server {
    links: Arc<Mutex<LinkStore>>,
    vm_bridge: Arc<Mutex<VMBridge>>,
    thread_handle: Option<JoinHandle<()>>,
    listening_port: Arc<Mutex<Option<u16>>>,
    control_tx: Sender<ServerCommand>,
    control_rx: Receiver<ServerCommand>,
}

impl Server {
    pub fn new(vm_bridge: Arc<Mutex<VMBridge>>) -> Self {
        let (tx, rx) = flume::unbounded();
        Self {
            links: Arc::new(Mutex::new(LinkStore::default())),
            vm_bridge,
            thread_handle: None,
            listening_port: Arc::new(Mutex::new(None)),
            control_tx: tx,
            control_rx: rx,
        }
    }

    pub fn generate_ref(&self) -> ServerRef {
        ServerRef {
            links: self.links.clone(),
            vm_bridge: self.vm_bridge.clone(),
            listening_port: self.listening_port.clone(),
        }
    }

    pub fn start_listening(&mut self, bind_addr: impl tokio::net::ToSocketAddrs + Send + 'static) {
        let s_ref = self.generate_ref();
        let rx = self.control_rx.clone();
        self.thread_handle = Some(thread::spawn(move || {
            let rt = runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async move {
                Self::listen_loop(bind_addr, s_ref, rx).await.unwrap();
            });
        }));
    }

    pub fn send_exa(&self, link: i16, queued_exa: Arc<Mutex<Option<PackedExa>>>) {
        //TODO: send exa -> usize, id in vm bridge
        // protocol send task -> take exa from vm bridge based on id
        // if exists send | else abort
        self.control_tx
            .send(ServerCommand::SendPackedExa(link, queued_exa))
            .unwrap();
    }

    pub fn connect(&self, addr: impl std::net::ToSocketAddrs) {
        let addrs = addr
            .to_socket_addrs()
            .unwrap()
            .collect::<Vec<SocketAddr>>()
            .into_boxed_slice();
        self.control_tx.send(ServerCommand::Connect(addrs)).unwrap();
    }

    pub fn wait(&mut self) {
        self.thread_handle.take().unwrap().join().unwrap();
    }

    async fn listen_loop(
        bind_addr: impl tokio::net::ToSocketAddrs,
        server_ref: ServerRef,
        control_rx: Receiver<ServerCommand>,
    ) -> Result<(), std::io::Error> {
        let listener = TcpListener::bind(bind_addr).await?;

        info!(
            "[net] server listening on {}",
            listener.local_addr().unwrap()
        );

        server_ref
            .set_listening_port_async(listener.local_addr().unwrap().port())
            .await;

        loop {
            tokio::select! {
                res = listener.accept() => {
                    let (stream, peer_addr) = res?;
                    info!("[net] new connection from {}", peer_addr);

                    let s_ref = server_ref.clone();
                    tokio::spawn(async move { protocol::respond(stream, s_ref).await });
                },
                res = control_rx.recv_async() => {
                    let command = match res {
                        Ok(c) => c,
                        Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "asd")),
                    };
                    match command {
                        ServerCommand::Connect(addrs) => {
                            let s_ref = server_ref.clone();
                            tokio::spawn(async move {
                                protocol::connect(&addrs[..], s_ref).await
                            });
                        },
                        ServerCommand::SendPackedExa(asd, pexa) => (),
                        ServerCommand::Exit => {
                            return Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "asd"))
                        }
                    }
                },
            }
        }
    }
}

enum ServerCommand {
    Connect(Box<[SocketAddr]>),
    SendPackedExa(i16, Arc<Mutex<Option<PackedExa>>>),
    Exit,
}

#[derive(Debug, Clone)]
struct ServerRef {
    links: Arc<Mutex<LinkStore>>,
    vm_bridge: Arc<Mutex<VMBridge>>,
    listening_port: Arc<Mutex<Option<u16>>>,
}

impl ServerRef {
    pub async fn get_port(&self) -> u16 {
        self.listening_port
            .lock()
            .await
            .as_ref()
            .unwrap()
            .to_owned()
    }

    pub async fn get_addr(&self, id: &i16) -> Option<SocketAddr> {
        self.links.lock().await.get_by_id(id).cloned()
    }

    pub async fn set_listening_port_async(&self, port: u16) {
        *self.listening_port.lock().await = Some(port);
    }

    pub async fn add_link_auto(&self, addr: SocketAddr, incoming: bool) -> i16 {
        let mut ls = self.links.lock().await;
        if ls.contains_addr(&addr) {
            return *ls.get_by_addr(&addr).unwrap();
        }
        for x in 1..=9999 {
            let k = if incoming { -(x as i16) } else { x as i16 };
            if !ls.contains_id(&k) {
                ls.insert(k, addr);
                return k;
            }
        }
        0
    }

    pub async fn add_link(&self, id: i16, addr: SocketAddr) -> Result<(), ()> {
        let mut ls = self.links.lock().await;
        if ls.contains_id(&id) || ls.contains_addr(&addr) {
            return Err(());
        }
        ls.insert(id, addr);
        Ok(())
    }
}

#[derive(Debug, Default)]
struct LinkStore {
    ids: HashMap<i16, SocketAddr>,
    addrs: HashMap<SocketAddr, i16>,
}

impl LinkStore {
    pub fn insert(&mut self, id: i16, addr: SocketAddr) {
        self.ids.insert(id, addr);
        self.addrs.insert(addr, id);
    }

    pub fn remove_by_id(&mut self, id: &i16) -> Option<SocketAddr> {
        let addr = self.ids.remove(id)?;
        self.addrs.remove(&addr);
        Some(addr)
    }

    pub fn remove_by_addr(&mut self, addr: &SocketAddr) -> Option<i16> {
        let id = self.addrs.remove(addr)?;
        self.ids.remove(&id);
        Some(id)
    }

    pub fn get_by_id(&self, id: &i16) -> Option<&SocketAddr> {
        self.ids.get(id)
    }

    pub fn get_by_addr(&self, addr: &SocketAddr) -> Option<&i16> {
        self.addrs.get(addr)
    }

    pub fn contains_id(&self, id: &i16) -> bool {
        self.ids.contains_key(id)
    }

    pub fn contains_addr(&self, addr: &SocketAddr) -> bool {
        self.addrs.contains_key(addr)
    }
}
