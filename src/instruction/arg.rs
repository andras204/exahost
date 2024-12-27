use std::{fmt::Display, str::FromStr};

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::exa::{status, ExaStatus};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum Arg {
    RegLabel(RegLabel),
    Number(i16),
    Comp(Comp),
    Keyword(Box<str>),
    JumpIndex(u8),
}

impl Arg {
    pub fn reg_label(&self) -> Result<RegLabel, ExaStatus> {
        match self {
            Self::RegLabel(r) => Ok(r.clone()),
            _ => Err(ExaStatus::Error(status::Error::InvalidArgument)),
        }
    }

    pub fn number(&self) -> Result<i16, ExaStatus> {
        match self {
            Self::Number(n) => Ok(*n),
            _ => Err(ExaStatus::Error(status::Error::InvalidArgument)),
        }
    }

    pub fn comp(&self) -> Result<Comp, ExaStatus> {
        match self {
            Self::Comp(c) => Ok(*c),
            _ => Err(ExaStatus::Error(status::Error::InvalidArgument)),
        }
    }

    pub fn jump_index(&self) -> Result<u8, ExaStatus> {
        match self {
            Self::JumpIndex(j) => Ok(*j),
            _ => Err(ExaStatus::Error(status::Error::InvalidArgument)),
        }
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
            a => write!(f, "{}", a),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum RegLabel {
    X,
    T,
    F,
    M,
    H(Box<str>),
}

impl FromStr for RegLabel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_uppercase()[..] {
            "X" => return Ok(Self::X),
            "T" => return Ok(Self::T),
            "F" => return Ok(Self::F),
            "M" => return Ok(Self::M),
            _ => (),
        }
        if s.starts_with('#') {
            Ok(Self::H(s.to_uppercase().into()))
        } else {
            Err(format!("cannot parse '{}' as RegisterLabel", s))
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum Comp {
    Eq,
    Gt,
    Lt,
    Ge,
    Le,
    Ne,
}

impl FromStr for Comp {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "=" => Ok(Self::Eq),
            ">" => Ok(Self::Gt),
            "<" => Ok(Self::Lt),
            ">=" => Ok(Self::Ge),
            "<=" => Ok(Self::Le),
            "!=" => Ok(Self::Ne),
            _ => Err(format!("cannot parse '{}' as Comparison", s)),
        }
    }
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
