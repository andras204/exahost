use std::net::ToSocketAddrs;
use flume::{Sender, Receiver};

use exavm::ExaVM;
use linker::LinkManager;
use exa::Exa;
use signal::HostSignal;

pub mod linker;
pub mod exavm;
pub mod lexar;
pub mod exa;
pub mod signal;

pub struct Host {
    host_name: String,
    link_manager: LinkManager,
    exa_vm: ExaVM,
    link_tx: Sender<HostSignal>,
    link_rx: Receiver<HostSignal>,
}

impl Host {
    pub fn new(host_name: &str, bind_addr: &str) -> Host {
        println!("Initializing host: {}", host_name);
        let mut link_manager = LinkManager::new(bind_addr);
        let (link_tx, link_rx) = link_manager.start_listening();
        let host = Host {
            host_name: host_name.to_string(),
            link_manager,
            exa_vm: ExaVM::new(),
            link_tx,
            link_rx,
        };
        host
    }

    pub fn add_exa(&mut self, exa: Exa) {
        self.exa_vm.add_exa(exa);
    }

    pub fn step(&mut self) {
        match self.exa_vm.step() {
            Some(l) => self.link_tx.send(l).unwrap(),
            None => (),
        }
        match self.link_rx.try_recv() {
            Ok(l) => match l {
                HostSignal::Link(link) => {
                    self.exa_vm.add_exa(link.1);
                },
                _ => (),
            },
            Err(_) => (),
        }
    }

    pub fn connect(&mut self, address: &(impl ToSocketAddrs + ?Sized)) {
        self.link_manager.connect(address);
    }
}
