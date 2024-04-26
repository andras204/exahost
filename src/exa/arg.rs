use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Arg {
    RegLabel(RegLabel),
    Number(i16),
    Comp(Comp),
    Keyword(Box<str>),
    JumpLabel(Box<str>),
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

impl Display for Arg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Comp(c) => write!(f, "{}", c),
            Self::Number(n) => write!(f, "{}", n),
            Self::Keyword(w) => write!(f, "{}", w),
            Self::RegLabel(r) => write!(f, "{}", r),
            Self::JumpLabel(l) => write!(f, "{}", l),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RegLabel {
    X,
    T,
    F,
    M,
    H(String),
}

impl Display for RegLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::X => write!(f, "X"),
            Self::T => write!(f, "T"),
            Self::F => write!(f, "F"),
            Self::M => write!(f, "M"),
            Self::H(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Comp {
    Eq,
    Gt,
    Lt,
    Ge,
    Le,
    Ne,
}

impl Display for Comp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eq => write!(f, "="),
            Self::Gt => write!(f, ">"),
            Self::Lt => write!(f, "<"),
            Self::Ge => write!(f, ">="),
            Self::Le => write!(f, "<="),
            Self::Ne => write!(f, "!="),
        }
    }
}
