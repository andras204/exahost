use std::collections::HashMap;

use once_cell::sync::Lazy;
use regex::RegexSet;

#[derive(Debug, Clone)]
pub struct Token {
    token: String,
    token_type: TokenType,
}

#[derive(Debug, Clone)]
pub enum TokenType {
    Instruction,
    Number,
    Register,
    Label,
    Comparison,
    Keyword,
}

impl Token {
    pub fn new(token: String, token_type: TokenType) -> Token {
        Token {
            token,
            token_type,
        }
    }
}

pub fn tokenize(instr: String) -> Result<Vec<Token>, String> {
    let sliced = split_instruction(instr);
    let tokens = parse_tokens(sliced);
    tokens
}

pub fn split_instruction(instr: String) -> Vec<String> {
    let mut sliced: Vec<String> = vec![ instr[..4].to_string() ];
    if instr.len() > 5 {
        let arg_slice = &instr[5..];
        let mut t_begin: usize = 0;
        let mut mid_word: bool = false;
        for x in 0..arg_slice.len() {
            match arg_slice.chars().nth(x).unwrap() {
                '\'' => {
                    mid_word = !mid_word;
                    if !mid_word {
                        sliced.push(arg_slice[t_begin..(x + 1)].to_string());
                        t_begin = x + 1;
                    }
                },
                ' ' => {
                    if !mid_word {
                        sliced.push(arg_slice[t_begin..x].to_string());
                        t_begin = x + 1;
                    }
                },
                _ => {},
            }
        }
        if arg_slice.len() > t_begin {
            sliced.push(arg_slice[t_begin..arg_slice.len()].to_string());
        }
    }
    sliced
}

pub fn parse_tokens(sliced: Vec<String>) -> Result<Vec<Token>, String> {
    let mut tokens: Vec<Token>;
    pattern_match(sliced[0])
}

/// regex match to determine `TokenType`s
pub fn pattern_match(str: String) -> Result<TokenType, String> {
    let mut args: Vec<Token> = Vec::with_capacity(3);
    static RS: Lazy<RegexSet> = Lazy::new(|| RegexSet::new([
        r"'*'",          // Keyword
        r"[A-Z]+[0-9]*", // Label
        r"[0-9]+",       // Number 
        r"[xtfm]{1}",    // Register
        r"[=!><]{1,2}",  // Comparison
    ]).unwrap());
    match RS.matches(&str[..]).into_iter().nth(0).unwrap() {
        0 => Ok(TokenType::Keyword),
        1 => Ok(TokenType::Label),
        2 => Ok(TokenType::Number),
        3 => Ok(TokenType::Register),
        4 => Ok(TokenType::Comparison),
        _ => Err(format!("Unknown argument at <line, arg index>")),
    }
}

/// match `TokenType`s with Instruction signatures
/// to catch type mismatch errors before execution
pub fn sig_match(tokens: &Vec<Token>) -> Result<(), String>{
    Ok(())
}
