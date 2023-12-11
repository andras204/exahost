use std::sync::mpsc::{Sender, Receiver, channel};

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
    link_tx: Sender<String>,
    link_rx: Receiver<String>,
    vm_tx: Sender<HostSignal>,
    vm_rx: Receiver<HostSignal>,
}

impl Host {
    pub fn new(host_name: &str, bind_addr: &str) -> Host {
        println!("Initializing host: {}", host_name);
        let (link_tx, link_rx) = channel();
        let (vm_tx, vm_rx) = channel();
        let mut host = Host {
            host_name: host_name.to_string(),
            link_manager: LinkManager::new(bind_addr.to_string(), link_tx.clone()),
            exa_vm: ExaVM::new(vm_tx.clone()),
            link_tx,
            link_rx,
            vm_tx,
            vm_rx,
        };

        host.start_listener();

        host
    }

    pub fn add_exa(&mut self, exa: Exa) {
        self.exa_vm.add_exa(exa);
    }

    pub fn step(&mut self) {
        self.exa_vm.step();
    }

    fn start_listener(&mut self) {
        self.link_manager.start_listening();
    }
}
