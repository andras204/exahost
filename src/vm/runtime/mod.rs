use std::{cell::RefCell, collections::HashMap, rc::Rc};

use fs::{FileHandle, FsModule};
use rand::{rngs::SmallRng, Rng, SeedableRng};

use crate::{
    exa::{
        status::{Block, Error, ExaStatus},
        Exa, Register,
    },
    hw_register::*,
};

pub mod fs;
pub mod reg_m;

#[derive(Debug)]
struct RuntimeInner {
    hostname: Box<str>,
    rng: SmallRng,

    fs: FsModule,
    hw: HashMap<Box<str>, Box<dyn HardwareRegister>>,
}

#[derive(Debug, Clone)]
pub struct Runtime {
    inner: Rc<RefCell<RuntimeInner>>,
    reg_m: Rc<RefCell<Option<Register>>>,
}

impl Runtime {
    pub fn new(hostname: &str, fs_root: &str) -> Self {
        let mut hw: HashMap<Box<str>, Box<dyn HardwareRegister>> = HashMap::new();
        let p = PrintRegister {};
        hw.insert(p.label_str(), Box::new(p));
        Self {
            inner: Rc::new(RefCell::new(RuntimeInner {
                hostname: hostname.into(),
                rng: SmallRng::from_os_rng(),
                fs: FsModule::new(fs_root),
                hw,
            })),
            reg_m: Rc::new(RefCell::new(None)),
        }
    }

    pub fn send_m(&self, value: Register) -> Result<(), ExaStatus> {
        let mut m = self.reg_m.borrow_mut();
        if m.is_none() {
            *m = Some(value);
            Ok(())
        } else {
            Err(ExaStatus::Block(Block::Send))
        }
    }

    pub fn recv_m(&self) -> Result<Register, ExaStatus> {
        let mut m = self.reg_m.borrow_mut();
        match m.take() {
            Some(r) => Ok(r),
            None => Err(ExaStatus::Block(Block::Recv)),
        }
    }

    pub fn would_m_read_not_block(&self) -> bool {
        self.reg_m.borrow().is_none()
    }

    pub fn hostname(&self) -> Register {
        Register::Keyword(self.inner.borrow().hostname.clone())
    }

    pub fn rand(&self, a: i16, b: i16) -> Register {
        let r = if a < b { a..=b } else { b..=a };
        Register::Number(self.inner.borrow_mut().rng.random_range(r))
    }

    pub fn make_file(&self) -> Option<FileHandle> {
        self.inner.borrow_mut().fs.make_file()
    }

    pub fn grab_file(&self, id: i16) -> Option<FileHandle> {
        self.inner.borrow_mut().fs.grab_file(id)
    }

    pub fn return_file(&self, fh: FileHandle) {
        self.inner.borrow_mut().fs.return_file(fh);
    }

    pub fn wipe_file(&self, id: i16) {
        self.inner.borrow_mut().fs.wipe_file(id);
    }

    pub fn hw_read(&self, exa: &Exa, label: Box<str>) -> Result<Register, ExaStatus> {
        match self.inner.borrow_mut().hw.get_mut(&label) {
            Some(hwr) => hwr.read(exa),
            None => Err(ExaStatus::Error(Error::InvalidHWRegisterAccess)),
        }
    }

    pub fn hw_write(&self, exa: &Exa, label: Box<str>, value: Register) -> Result<(), ExaStatus> {
        match self.inner.borrow_mut().hw.get_mut(&label) {
            Some(hwr) => hwr.write(exa, value),
            None => Err(ExaStatus::Error(Error::InvalidHWRegisterAccess)),
        }
    }
}
