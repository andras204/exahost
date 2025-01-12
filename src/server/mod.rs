use std::{
    collections::HashMap,
    sync::Arc,
    thread::{self, JoinHandle},
};

use flume::Sender;
use link::LinkHandle;
use log::info;
use tokio::{
    net::{TcpListener, ToSocketAddrs},
    runtime::{self, Runtime},
    sync::Mutex,
};

use crate::exa::PackedExa;

pub mod link;
pub mod link_actions;

pub struct Server {
    link_handles: Arc<Mutex<HashMap<i16, LinkHandle>>>,
    thread_handle: Option<JoinHandle<()>>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            link_handles: Arc::new(Mutex::new(HashMap::new())),
            thread_handle: None,
        }
    }

    pub fn start(&mut self, bind_addr: impl ToSocketAddrs + Send + 'static) {
        let lhs = self.link_handles.clone();
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
        link_handles: Arc<Mutex<HashMap<i16, LinkHandle>>>,
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

            tokio::spawn(async move {
                let (mut l, h) = link::wrap_tcp(stream);
                {
                    let mut lh = link_handles.lock().await;
                    let k = *(lh.keys().max().unwrap_or(&0));
                    lh.insert(k, h);
                }
                l.handle_connection().await;
            });
        }
    }
}
