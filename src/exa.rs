use serde::{Deserialize, Serialize};

use crate::file::File;

mod arg;
mod instruction;
mod register;

pub use arg::{Arg, Comp, RegLabel};
pub use instruction::{Instruction, OpCode};
pub use register::Register;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Exa {
    pub name: String,
    pub instr_list: Box<[Instruction]>,
    pub instr_ptr: u8,
    pub repl_counter: usize,
    pub reg_x: Register,
    pub reg_t: Register,
    pub reg_f: Option<(i16, File)>,
}

impl Exa {
    pub fn new(name: &str, instr_list: Box<[Instruction]>) -> Self {
        Self {
            name: name.to_string(),
            instr_list,
            instr_ptr: 0,
            repl_counter: 0,
            reg_x: Register::Number(0),
            reg_t: Register::Number(0),
            reg_f: None,
        }
    }
}
