use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::exa::{Instruction, Register};
use crate::runtime::fs::File;
use crate::runtime::SharedRT;

use super::Exa;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub struct PackedExa {
    pub name: String,
    pub instr_list: Box<[Instruction]>,
    pub instr_ptr: u8,
    pub repl_counter: u16,
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
            reg_x: Register::zero(),
            reg_t: Register::zero(),
            reg_f: None,
        }
    }

    pub fn hydrate(self, rt_ref: SharedRT) -> Exa {
        Exa {
            name: self.name,
            instr_list: self.instr_list,
            instr_ptr: self.instr_ptr,
            repl_counter: self.repl_counter,
            reg_x: self.reg_x,
            reg_t: self.reg_t,
            reg_f: self.reg_f,
            reg_m: rt_ref.get_default_reg_m(),
            rt_ref,
        }
    }
}
