use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

mod arg;
mod file;
mod instruction;
mod register;

pub use arg::{Arg, Comp, RegLabel};
pub use file::File;
pub use instruction::{Instruction, OpCode};
pub use register::Register;

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Exa {
    pub name: String,
    pub instr_list: Box<[Instruction]>,
    pub instr_ptr: u8,
    pub repl_counter: usize,
    pub reg_x: Register,
    pub reg_t: Register,
    pub reg_f: Option<(i16, File)>,
    pub mode: RegMMode,
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
            mode: RegMMode::Local,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum RegMMode {
    Global,
    Local,
}
