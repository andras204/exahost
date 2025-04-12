use std::net::{SocketAddr, ToSocketAddrs};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    bind_address: SocketAddr,
    background_threads: usize,
}

impl ServerConfig {
    pub fn new(bind_addr: impl ToSocketAddrs, bg_threads: usize) -> Self {
        Self {
            bind_address: bind_addr.to_socket_addrs().unwrap().last().unwrap(),
            background_threads: bg_threads,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:6800".to_socket_addrs().unwrap().last().unwrap(),
            background_threads: 2,
        }
    }
}
