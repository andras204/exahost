use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Token {
    pub row: usize,
    pub col: usize,
    pub content: String,
    pub ttype: TokenType,
}

impl Token {
    pub fn new<T>(row: usize, col: usize, content: T, ttype: TokenType) -> Self
    where
        T: Display,
    {
        Self {
            row,
            col,
            content: content.to_string(),
            ttype,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    OpCode,
    Number,
    RegisterLabel,
    JumpLabel,
    Comparison,
    Keyword,
    MacroStart,
    MacroEnd,
    MacroReplace,
    Comment,
}
