use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::exa::{Instruction, Register};
use crate::runtime::fs::File;
use crate::runtime::RuntimeHarness;

use super::Exa;

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PackedExa {
    pub name: String,
    pub instr_list: Box<[Instruction]>,
    pub instr_ptr: u8,
    pub repl_counter: usize,
    pub reg_x: Register,
    pub reg_t: Register,
    pub reg_f: Option<(i16, File)>,
}

impl PackedExa {
    pub fn new(name: &str, instr_list: Box<[Instruction]>) -> Self {
        Self {
            name: name.into(),
            instr_list,
            instr_ptr: 0,
            repl_counter: 0,
            reg_x: Register::default(),
            reg_t: Register::default(),
            reg_f: None,
        }
    }

    pub fn hydrate(self, harness: RuntimeHarness) -> Exa {
        Exa {
            name: self.name,
            instr_list: self.instr_list,
            instr_ptr: self.instr_ptr,
            repl_counter: self.repl_counter,
            reg_x: self.reg_x,
            reg_t: self.reg_t,
            reg_f: self.reg_f,
            harness,
        }
    }
}
