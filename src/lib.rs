use std::net::ToSocketAddrs;
use flume::Receiver;

use exavm::ExaVM;
use compiler::Compiler;
use linker::LinkManager;
use exa::Exa;

use crate::compiler::CompilerConfig;

pub mod linker;
pub mod exavm;
pub mod exa;
pub mod compiler;

#[derive(Debug, Clone)]
pub enum HostSignal {
    Link((i16, Exa)),
    Step,
    Stop,
}

pub struct Host {
    host_name: String,
    link_manager: LinkManager,
    exa_vm: ExaVM,
    exa_compiler: Compiler,
    link_rx: Receiver<HostSignal>,
}

impl Host {
    pub fn new(host_name: &str, bind_addr: &str) -> Host {
        println!("Initializing host: {}", host_name);
        let mut link_manager = LinkManager::new(bind_addr);
        let link_rx = link_manager.start_listening();
        let exa_compiler = Compiler::with_config(CompilerConfig::extended());
        let host = Host {
            host_name: host_name.to_string(),
            link_manager,
            exa_vm: ExaVM::new(),
            exa_compiler,
            link_rx,
        };
        host
    }

    pub fn compile_exa(&self, name: &str, instructions: Vec<String>) -> Result<Exa, Vec<String>> {
        let instr = self.exa_compiler.compile(instructions)?;
        Ok(Exa::new(name, instr))
    }

    pub fn add_exa(&mut self, exa: Exa) {
        self.exa_vm.add_exa(exa);
    }

    pub fn step(&mut self) {
        match self.link_rx.try_recv() {
            Ok(l) => match l {
                HostSignal::Link(link) => {
                    self.exa_vm.add_exa(link.1);
                },
                _ => (),
            },
            Err(_) => (),
        }
        for lrq in self.exa_vm.step() {
            match self.link_manager.queue(lrq) {
                Ok(_) => (),
                Err(e) => eprintln!("[VM]: Error: {}", e),
            }
        }
        self.link_manager.send();
    }

    pub fn connect(&mut self, address: &(impl ToSocketAddrs + ?Sized)) {
        self.link_manager.connect(address);
    }
}
