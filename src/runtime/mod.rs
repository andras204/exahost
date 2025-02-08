use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Arc, Mutex, Weak},
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

#[derive(Debug)]
pub struct ArcRT {
    rt: Arc<Runtime>,
}

impl ArcRT {
    pub fn generate_ref(&self) -> WeakRT {
        WeakRT {
            rt: Arc::downgrade(&self.rt),
        }
    }
}

impl Deref for ArcRT {
    type Target = Runtime;

    fn deref(&self) -> &Self::Target {
        &self.rt
    }
}

#[derive(Debug, Clone)]
pub struct WeakRT {
    rt: Weak<Runtime>,
}

impl WeakRT {
    pub fn hostname(&self) -> Register {
        self.rt.upgrade().unwrap().hostname()
    }

    pub fn rand(&self, a: i16, b: i16) -> Register {
        self.rt.upgrade().unwrap().rand(a, b)
    }

    pub fn make_file(&self) -> Option<FileHandle> {
        self.rt.upgrade().unwrap().make_file()
    }

    pub fn grab_file(&self, id: i16) -> Option<FileHandle> {
        self.rt.upgrade().unwrap().grab_file(id)
    }

    pub fn return_file(&self, fh: FileHandle) {
        self.rt.upgrade().unwrap().return_file(fh)
    }

    pub fn wipe_file(&self, id: i16) {
        self.rt.upgrade().unwrap().wipe_file(id)
    }

    pub fn dial(&self, channel: i16) -> ChannelHandle {
        self.rt.upgrade().unwrap().dial(channel)
    }

    pub fn get_default_reg_m(&self) -> ChannelHandle {
        self.rt.upgrade().unwrap().get_default_reg_m()
    }

    pub fn hw_read(&self, exa: &Exa, label: Box<str>) -> Result<Register, ExaStatus> {
        self.rt.upgrade().unwrap().hw_read(exa, label)
    }

    pub fn hw_write(&self, exa: &Exa, label: Box<str>, value: Register) -> Result<(), ExaStatus> {
        self.rt.upgrade().unwrap().hw_write(exa, label, value)
    }
}

#[derive(Debug)]
pub struct Runtime {
    hostname: Box<str>,
    rng: Mutex<ThreadRng>,

    ipc: Mutex<IpcModule>,
    fs: Mutex<FsModule>,
    hw: Mutex<HashMap<Box<str>, Box<dyn HardwareRegister>>>,
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
            rng: Mutex::new(thread_rng()),
            ipc: Mutex::new(ipc),
            fs: Mutex::new(FsModule::new("./files")),
            hw: Mutex::new(hw_map),
        }
    }

    pub fn make_shared(self) -> ArcRT {
        ArcRT { rt: Arc::new(self) }
    }

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

    pub fn dial(&self, channel: i16) -> ChannelHandle {
        self.ipc.lock().unwrap().dial(channel)
    }

    pub fn get_default_reg_m(&self) -> ChannelHandle {
        self.ipc.lock().unwrap().get_default_channel()
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
