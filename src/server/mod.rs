use std::{
    net::{SocketAddr, ToSocketAddrs},
    thread::{self, JoinHandle},
};

use log::{error, info, warn};
use tokio::net::TcpListener;

use crate::backbone::Backbone;

mod protocol;
mod tasks;

#[derive(Debug)]
pub struct Server {
    thread_handle: Option<JoinHandle<Result<(), std::io::Error>>>,

    bind_addr: SocketAddr,
    worker_threads: usize,

    backbone: Backbone,
}

impl Server {
    pub fn new(
        backbone: Backbone,
        bind_address: &impl ToSocketAddrs,
        worker_threads: usize,
    ) -> Self {
        Self {
            thread_handle: None,
            bind_addr: bind_address.to_socket_addrs().unwrap().last().unwrap(),
            worker_threads,
            backbone,
        }
    }

    pub fn start(&mut self) -> Result<(), std::io::Error> {
        let addr = self.bind_addr.clone();
        let rt = if self.worker_threads == 0 {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
        } else if self.worker_threads == 1 {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
        } else {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(self.worker_threads)
                .enable_all()
                .build()
                .unwrap()
        };

        let listener = rt.block_on(async move { TcpListener::bind(addr).await })?;

        self.backbone
            .set_server_listening_addr(listener.local_addr().unwrap());

        let bb = self.backbone.clone();

        self.thread_handle = Some(thread::spawn(move || {
            rt.block_on(async move { Self::main_loop(bb, listener).await })
        }));
        Ok(())
    }

    async fn main_loop(backbone: Backbone, listener: TcpListener) -> Result<(), std::io::Error> {
        let mut shutdown = backbone.get_shutdown_listener();
        info!("[SERVER] entering main loop");
        loop {
            tokio::select! {
                res = listener.accept() => {
                    let (tcp, addr) = match res {
                        Ok(v) => v,
                        Err(_) => break,
                    };
                    info!("[SERVER] incoming request from {}", addr);
                    let bb = backbone.clone();
                    tokio::spawn(async move { tasks::handle_request(bb, tcp).await });
                }
                comm = backbone.server_command_rx().recv_async() => {
                    let comm = match comm {
                        Ok(c) => c,
                        Err(_) => break,
                    };
                    let bb = backbone.clone();
                    tokio::spawn(async move { tasks::exec_server_command(bb, comm).await });
                }
                _ = shutdown.recv() => {
                    break;
                }
            }
        }
        info!("[SERVER] shutting down");
        Ok(())
    }
}
