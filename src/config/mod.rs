mod compiler_config;
mod server_config;

use std::net::{SocketAddr, ToSocketAddrs};

pub use compiler_config::CompilerConfig;
use serde::{Deserialize, Serialize};
pub use server_config::ServerConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    hostname: Box<str>,
    max_exas: usize,
    server: ServerConfig,
    compiler: CompilerConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hostname: "Rhizome".into(),
            max_exas: 9,
            server: ServerConfig::default(),
            compiler: CompilerConfig::extended(),
        }
    }
}
