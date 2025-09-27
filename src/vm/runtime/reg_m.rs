use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crate::exa::Register;

pub struct RegM {
    read_queue: VecDeque<Rc<RefCell<MRead>>>,
    write_queue: VecDeque<Rc<RefCell<MWrite>>>,
}

pub enum Mop {
    Write(MWrite),
    Read(MRead),
}

impl Mop {
    pub fn new_read() -> Self {
        Self::Read(MRead(None))
    }
}

struct MRead(Option<Register>);
struct MWrite(Option<Register>);
