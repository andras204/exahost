use std::{cmp::Ordering, str::FromStr};

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
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

pub enum ParseError {
    StringTooLong,
    NumberOutOfBounds,
}

impl FromStr for Register {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > 256 {
            return Err(ParseError::StringTooLong);
        }
        match s.parse::<i16>() {
            Ok(n) => {
                if (-9999..=9999).contains(&n) {
                    Ok(Register::Number(n))
                } else {
                    Err(ParseError::NumberOutOfBounds)
                }
            }
            Err(_) => Ok(Register::Keyword(s.into())),
        }
    }
}
