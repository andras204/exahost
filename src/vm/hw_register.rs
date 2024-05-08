use crate::{exa::Register, vm::ExaResult};

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// enum Error {
//     ReadError,
//     WriteError,
// }

pub trait HWRegister {
    fn read(&self) -> Result<Register, ExaResult> {
        eprintln!("Hardware register is write only!");
        Err(ExaResult::Error(
            crate::vm::RuntimeError::InvalidHWRegAccess,
        ))
    }
    fn write(&mut self, val: Register) -> Result<(), ExaResult> {
        eprintln!("Hardware register is read only!");
        Err(ExaResult::Error(
            crate::vm::RuntimeError::InvalidHWRegAccess,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct DebugOutput {
    pub prefix: Box<str>,
}

impl HWRegister for DebugOutput {
    fn write(&mut self, val: Register) -> Result<(), ExaResult> {
        println!("{} {}", self.prefix, val);
        Ok(())
    }
}
