use std::sync::{Arc, Mutex};

use ipc::IpcChannel;
use rand::{rngs::ThreadRng, thread_rng};

use self::{fs::FsModule, ipc::IpcModule, net::NetModule};

pub mod fs;
pub mod ipc;
pub mod net;

#[derive(Debug, Clone)]
pub struct RuntimeHarness {
    hostname: Box<str>,
    reg_m: IpcChannel,
    rng: Arc<Mutex<ThreadRng>>,

    ipc: Arc<Mutex<IpcModule>>,
    fs: Arc<Mutex<FsModule>>,
    net: Arc<Mutex<NetModule>>,
    hw: (),
}

impl RuntimeHarness {
    pub fn new(hostname: String) -> Self {
        let ipc = IpcModule::new();
        Self {
            hostname: hostname.into_boxed_str(),
            reg_m: ipc.get_default_channel(),
            rng: Arc::new(Mutex::new(thread_rng())),
            ipc: Arc::new(Mutex::new(ipc)),
            fs,
            net,
            hw: (),
        }
    }
}
