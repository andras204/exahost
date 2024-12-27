use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use fs::FileHandle;
use hw_register::{HardwareRegister, PrintRegister};
use ipc::ChannelHandle;
use rand::{rngs::ThreadRng, thread_rng, Rng};

use crate::exa::{Exa, ExaStatus, Register};

use self::{fs::FsModule, ipc::IpcModule};

pub mod fs;
pub mod hw_register;
pub mod ipc;
pub mod net;

#[derive(Debug, Clone)]
pub struct Runtime {
    hostname: Box<str>,
    rng: Arc<Mutex<ThreadRng>>,

    ipc: Arc<Mutex<IpcModule>>,
    fs: Arc<Mutex<FsModule>>,
    hw: Arc<Mutex<HashMap<Box<str>, Box<dyn HardwareRegister>>>>,
}

impl Runtime {
    pub fn new(hostname: &str) -> Self {
        let ipc = IpcModule::new();

        let mut hw_map: HashMap<Box<str>, Box<dyn HardwareRegister>> = HashMap::new();
        let prnt_reg = PrintRegister {};
        hw_map.insert(
            prnt_reg.label_str().to_uppercase().into(),
            Box::new(prnt_reg),
        );

        Self {
            hostname: hostname.into(),
            rng: Arc::new(Mutex::new(thread_rng())),
            ipc: Arc::new(Mutex::new(ipc)),
            fs: Arc::new(Mutex::new(FsModule::new("./files"))),
            hw: Arc::new(Mutex::new(hw_map)),
        }
    }

    pub fn get_harness(&self) -> RuntimeHarness {
        RuntimeHarness {
            hostname: self.hostname.clone(),
            reg_m: self.ipc.lock().unwrap().get_default_channel(),
            rng: self.rng.clone(),
            ipc: self.ipc.clone(),
            fs: self.fs.clone(),
            hw: self.hw.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeHarness {
    hostname: Box<str>,
    reg_m: ChannelHandle,
    rng: Arc<Mutex<ThreadRng>>,

    ipc: Arc<Mutex<IpcModule>>,
    fs: Arc<Mutex<FsModule>>,
    hw: Arc<Mutex<HashMap<Box<str>, Box<dyn HardwareRegister>>>>,
}

impl RuntimeHarness {
    pub fn hostname(&self) -> Register {
        Register::Keyword(self.hostname.clone())
    }

    pub fn rand(&self, a: i16, b: i16) -> Register {
        let range = if a < b { a..=b } else { b..=a };
        Register::Number(self.rng.lock().unwrap().gen_range(range))
    }

    pub fn make_file(&self) -> Option<FileHandle> {
        self.fs.lock().unwrap().make_file()
    }

    pub fn grab_file(&self, id: i16) -> Option<FileHandle> {
        self.fs.lock().unwrap().grab_file(id)
    }

    pub fn return_file(&self, fh: FileHandle) {
        self.fs.lock().unwrap().return_file(fh);
    }

    pub fn wipe_file(&self, id: i16) {
        self.fs.lock().unwrap().wipe_file(id);
    }

    pub fn send(&self, value: Register) -> Result<(), ExaStatus> {
        let mut reg_m = self.reg_m.1.lock().unwrap();
        if reg_m.is_none() {
            *reg_m = Some(value);
            Ok(())
        } else {
            Err(ExaStatus::Block(crate::exa::Block::Send))
        }
    }

    pub fn recv(&self) -> Result<Register, ExaStatus> {
        match self.reg_m.1.lock().unwrap().take() {
            Some(r) => Ok(r),
            None => Err(ExaStatus::Block(crate::exa::Block::Recv)),
        }
    }

    pub fn is_m_read_non_block(&self) -> bool {
        self.reg_m.1.lock().unwrap().is_some()
    }

    pub fn dial(&mut self, channel: i16) {
        self.reg_m = self.ipc.lock().unwrap().dial(channel);
    }

    pub fn channel_id(&self) -> Register {
        Register::Number(self.reg_m.0)
    }

    pub fn hw_read(&self, exa: &Exa, label: Box<str>) -> Result<Register, ExaStatus> {
        match self.hw.lock().unwrap().get_mut(&label) {
            Some(hwr) => hwr.read(exa),
            None => Err(ExaStatus::Error(crate::exa::Error::InvalidHWRegisterAccess)),
        }
    }

    pub fn hw_write(&self, exa: &Exa, label: Box<str>, value: Register) -> Result<(), ExaStatus> {
        match self.hw.lock().unwrap().get_mut(&label) {
            Some(hwr) => hwr.write(exa, value),
            None => Err(ExaStatus::Error(crate::exa::Error::InvalidHWRegisterAccess)),
        }
    }
}
