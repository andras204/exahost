use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt::Display};
use strum::{Display, EnumString};

use crate::file::File;

pub type InstrTuple = (Instruction, Option<Arg>, Option<Arg>, Option<Arg>);

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[strum(serialize_all = "lowercase")]
pub enum Instruction {
    Copy,
    Void,

    Addi,
    Subi,
    Muli,
    Divi,
    Modi,
    Swiz,

    Test,
    TestMrd,
    TestEof,

    Mark,
    Jump,
    Fjmp,
    Tjmp,

    Make,
    Grab,
    File,
    Seek,
    Drop,
    Wipe,

    Link,
    Repl,
    Halt,
    Kill,

    Rand,
    Host,

    Noop,
    Prnt,
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
pub enum Arg {
    RegLabel(RegLabel),
    Number(i16),
    Comp(Comp),
    Keyword(String),
    JumpLabel(String),
}

impl Arg {
    pub fn reg_label(&self) -> Result<RegLabel, &str> {
        match self {
            Self::RegLabel(r) => Ok(r.clone()),
            _ => Err("arg is not register label"),
        }
    }

    pub fn number(&self) -> Result<i16, &str> {
        match self {
            Self::Number(n) => Ok(*n),
            _ => Err("arg is not number"),
        }
    }

    pub fn comp(&self) -> Result<Comp, &str> {
        match self {
            Self::Comp(c) => Ok(*c),
            _ => Err("arg is not comparison"),
        }
    }

    pub fn jump_label(&self) -> Result<String, &str> {
        match self {
            Self::JumpLabel(l) => Ok(l.to_string()),
            _ => Err("arg is not label"),
        }
    }

    pub fn reg_t() -> Self {
        Arg::RegLabel(RegLabel::T)
    }

    pub fn is_reg_m(&self) -> bool {
        match self {
            Arg::RegLabel(r) => matches!(r, RegLabel::M),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RegLabel {
    X,
    T,
    F,
    M,
    H(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Comp {
    Eq,
    Gt,
    Lt,
    Ge,
    Le,
    Ne,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Register {
    Number(i16),
    Keyword(String),
}

impl Display for Register {
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
