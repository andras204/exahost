use super::Token;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Error {
    pub row: usize,
    pub col: usize,
    pub context: String,
    pub etype: ErrorType,
}

impl Error {
    pub fn new<T>(row: usize, col: usize, content: T, etype: ErrorType) -> Self
    where
        T: Display,
    {
        Self {
            row,
            col,
            context: content.to_string(),
            etype,
        }
    }

    pub fn from_token(t: Token, etype: ErrorType) -> Self {
        Self {
            row: t.row,
            col: t.col,
            context: t.content,
            etype,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    UnknownInstruction,
    NumberOutOfRange,
    NestedMacros,
    MissingRepTag,
    MissingEndTag,
    InvalidNumber,
    NotOpCode,
    NotArg,
    ArgTypeMismatch,
    SigLenMismatch,
    DuplicateLabel,
    UndefinedLabel,
}
