use std::sync::Mutex;

use rand::{rngs::ThreadRng, thread_rng};

use self::{fs::FsModule, ipc::IpcModule, net::NetModule};

pub mod fs;
pub mod ipc;
pub mod net;

#[derive(Debug)]
pub struct RuntimeHarness {
    hostname: Box<str>,
    rng: Mutex<ThreadRng>,
    ipc: Option<IpcModule>,
    fs: Option<FsModule>,
    net: Option<NetModule>,
    hw: Option<()>,
}

impl RuntimeHarness {
    pub fn new(
        hostname: String,
        ipc: Option<IpcModule>,
        fs: Option<FsModule>,
        net: Option<NetModule>,
        hw: Option<()>,
    ) -> Self {
        Self {
            hostname: hostname.into_boxed_str(),
            rng: Mutex::new(thread_rng()),
            ipc,
            fs,
            net,
            hw,
        }
    }
}

impl Default for RuntimeHarness {
    fn default() -> Self {
        Self {
            hostname: "<unknown>".into(),
            rng: Mutex::new(thread_rng()),
            ipc: None,
            fs: None,
            net: None,
            hw: None,
        }
    }
}
