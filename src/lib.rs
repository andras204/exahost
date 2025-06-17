use std::net::ToSocketAddrs;

use backbone::Backbone;
use cli::Cli;
use compiler::Compiler;
use server::Server;
use vm::{runtime::Runtime, VM};

pub mod backbone;
pub mod cli;
pub mod compiler;
pub mod config;
pub mod exa;
pub mod hw_register;
pub mod server;
mod tests;
pub mod vm;

#[derive(Debug)]
pub struct Host {
    backbone: Backbone,
    vm: VM,
    server: Server,
    cli: Cli,
}

impl Host {
    pub fn new(
        hostname: &str,
        max_exas: usize,
        bind_address: &impl ToSocketAddrs,
        server_worker_threads: usize,
    ) -> Self {
        let compiler = Compiler::new(config::CompilerConfig::extended());
        let backbone = Backbone::new(hostname, max_exas, compiler);
        let rt = Runtime::new(hostname, "./files");
        let vm = VM::new(rt, backbone.clone());
        let server = Server::new(backbone.clone(), bind_address, server_worker_threads);
        let cli = Cli::new(backbone.clone());
        Self {
            backbone,
            vm,
            server,
            cli,
        }
    }

    pub fn start(&mut self) -> Result<(), std::io::Error> {
        self.server.start()?;
        self.cli.start();
        Ok(())
    }
}
