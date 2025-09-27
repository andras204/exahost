use std::{net::ToSocketAddrs, thread, thread::JoinHandle};

use backbone::Backbone;
use cli::Cli;
use compiler::Compiler;
use log::info;
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
    server: Server,
    cli: Cli,
    vm_thread: Option<JoinHandle<()>>,
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
        let server = Server::new(backbone.clone(), bind_address, server_worker_threads);
        let cli = Cli::new(backbone.clone());
        Self {
            backbone,
            server,
            cli,
            vm_thread: None,
        }
    }

    pub fn start(&mut self) -> Result<(), std::io::Error> {
        self.server.start()?;
        let bb = self.backbone.clone();
        self.vm_thread = Some(thread::spawn(move || {
            let rt = Runtime::new(bb.hostname(), "./files");
            let mut vm = VM::new(rt, bb);
            vm.main_loop();
        }));
        self.cli.start();
        Ok(())
    }
}
