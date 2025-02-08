use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    thread::{self, JoinHandle},
};

use log::info;
use tokio::{
    net::{TcpListener, ToSocketAddrs},
    runtime::{self},
    sync::Mutex,
};

use crate::{exa::PackedExa, runtime::WeakRT};

pub mod link;
pub mod link_actions;
mod protocol;

pub struct Server {
    links: Arc<Mutex<HashMap<i16, SocketAddr>>>,
    thread_handle: Option<JoinHandle<()>>,
    rt_ref: WeakRT,
}

impl Server {
    pub fn new(rt_ref: WeakRT) -> Self {
        Self {
            links: Arc::new(Mutex::new(HashMap::new())),
            thread_handle: None,
            rt_ref,
        }
    }

    pub fn start(&mut self, bind_addr: impl ToSocketAddrs + Send + 'static) {
        let lhs = self.links.clone();
        self.thread_handle = Some(thread::spawn(move || {
            let rt = runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async move {
                Self::listen_loop(lhs, bind_addr).await.unwrap();
            });
        }));
    }

    pub fn wait(&mut self) {
        self.thread_handle.take().unwrap().join().unwrap();
    }

    async fn listen_loop(
        link_handles: Arc<Mutex<HashMap<i16, SocketAddr>>>,
        bind_addr: impl ToSocketAddrs,
    ) -> Result<(), std::io::Error> {
        let listener = TcpListener::bind(bind_addr).await?;
        info!(
            "[net] server listening on {}",
            listener.local_addr().unwrap()
        );
        loop {
            let (stream, peer_addr) = listener.accept().await?;

            info!("[net] new connection accepted from {}", peer_addr);

            let link_handles = link_handles.clone();

            tokio::spawn(async move { protocol::handle_connection() });
        }
    }
}
