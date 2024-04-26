use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::file::File;

mod arg;
mod instruction;

pub use arg::{Arg, Comp, RegLabel};
pub use instruction::{InstrTuple, Instruction};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Exa {
    pub name: String,
    pub instr_list: Box<[InstrTuple]>,
    pub instr_ptr: u8,
    pub repl_counter: usize,
    pub reg_x: Register,
    pub reg_t: Register,
    pub reg_f: Option<(i16, File)>,
}

impl Exa {
    pub fn new(name: &str, instr_list: Box<[InstrTuple]>) -> Self {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Register {
    Number(i16),
    Keyword(Box<str>),
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::Keyword(w) => write!(f, "{}", w),
        }
    }
}

impl PartialOrd for Register {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            Self::Number(n) => match other {
                Self::Number(m) => Some(n.cmp(m)),
                Self::Keyword(_) => None,
            },
            Self::Keyword(k) => match other {
                Self::Number(_) => None,
                Self::Keyword(w) => Some(k.cmp(w)),
            },
        }
    }
}
