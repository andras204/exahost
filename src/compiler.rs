use std::{collections::HashMap, fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::exa::{Arg, Comp, RegLabel, Instruction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgType {
    Number,
    Keyword,
    Register,
    Label,
    Comparison,
}

// split this into internal and external types
// internal type is enum
// external is pretty
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilerError {
    UnknownInstruction(String),
    InstructionNotAllowed(Instruction),
    CommentParseError,
    ArgTypeMismatch(String, ArgType),
    NumberOutOfBounds,
    SignatureMismatch(Instruction, u8, u8),
    InvalidComparison(String),
}

impl Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_msg = match self {
            Self::UnknownInstruction(s) => format!("unknown instruction '{}'", s),
            Self::InstructionNotAllowed(i) => format!("use of {:?} disallowed by compiler configuration", i),
            Self::CommentParseError => "comments cannot be parsed".to_string(),
            Self::ArgTypeMismatch(s, t) => format!("cannot convert '{}' to {:?}", s, t),
            Self::NumberOutOfBounds => "number too large or too small".to_string(),
            Self::SignatureMismatch(i, s, g) => format!("{:?} requires {} arguments, found {}", i, s, g),
            Self::InvalidComparison(s) => format!("invalid comparison operator '{}'", s),
        };
        write!(f, "{}", err_msg)
    }
}

impl FromStr for Instruction {
    type Err = CompilerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "copy" => Ok(Self::Copy),

            "addi" => Ok(Self::Addi),
            "subi" => Ok(Self::Subi),
            "muli" => Ok(Self::Muli),
            "divi" => Ok(Self::Divi),
            "modi" => Ok(Self::Modi),
            "swiz" => Ok(Self::Swiz),

            "test" => Ok(Self::Test),

            "mark" => Ok(Self::Mark),
            "jump" => Ok(Self::Jump),
            "fjmp" => Ok(Self::Fjmp),
            "tjmp" => Ok(Self::Tjmp),

            "link" => Ok(Self::Link),
            "repl" => Ok(Self::Repl),
            "halt" => Ok(Self::Halt),
            "kill" => Ok(Self::Kill),

            "noop" => Ok(Self::Noop),
            "prnt" => Ok(Self::Prnt),
            _ => Err(CompilerError::UnknownInstruction(s.to_string())),
        } 
    }
}

#[derive(Debug, Clone)]
pub struct Compiler {
    instruction_signatures: HashMap<Instruction, Vec<Vec<ArgType>>>,
    comparisons: Vec<String>,
    allow_multi_m: bool,
    keyword_delimiter: char,
    comment_prefixes: Vec<String>,
}

impl Compiler {
    pub fn new() -> Self {
        let config = CompilerConfig::default();
        Self {
            instruction_signatures: config.generate_signatures(),
            comparisons: config.generate_comparisons(),
            allow_multi_m: config.allow_multi_m,
            keyword_delimiter: config.keyword_delimiter,
            comment_prefixes: config.comment_prefixes,
        }
    }

    pub fn with_config(config: CompilerConfig) -> Self {
        Self {
            instruction_signatures: config.generate_signatures(),
            comparisons: config.generate_comparisons(),
            allow_multi_m: config.allow_multi_m,
            keyword_delimiter: config.keyword_delimiter,
            comment_prefixes: config.comment_prefixes,
        }
    }

    pub fn compile(&self, source: Vec<String>) -> Result<Vec<(Instruction, Option<Vec<Arg>>)>, Vec<String>> {
        let mut errs: Vec<String> = Vec::new();
        let mut compiled: Vec<(Instruction, Option<Vec<Arg>>)> = Vec::with_capacity(source.len());
        if source.len() == 0 { errs.push("nothing to compile".to_string()); }
        for x in 0..source.len() {
            let split = match self.split_line(&source[x]) {
                Ok(s) => s,
                Err(e) => {
                    errs.push(format!("Error on line {}: {}", x, e));
                    continue;
                },
            };
            let instr = match self.parse_line(split) {
                Ok(i) => i,
                Err(e) => {
                    errs.push(format!("Error on line {}: {}", x, e));
                    continue;
                },
            };
            // check for multi M use
            if !self.allow_multi_m {
                let mut ms = 0;
                match instr.1.clone() {
                    Some(args) => {
                        for a in args {
                            if a == Arg::Register(RegLabel::M) { ms += 1; }
                        }
                    },
                    None => (),
                }
                if ms > 1 {
                    errs.push(format!("Error on line {}: multiple M use not allowed", x));
                    continue;
                }
            }
            compiled.push(instr);
        }
        if errs.len() > 0 { return Err(errs); }
        Ok(compiled)
    }

    fn split_line(&self, line: &String) -> Result<Vec<String>, CompilerError> {
        // unify comments
        for c in self.comment_prefixes.clone() {
            if line.starts_with(&c) {
                return Ok(vec!["note".to_string()]);
            }
        }
        
        // filter out short lines
        if line.len() < 4 {
            return Err(CompilerError::UnknownInstruction(line.to_owned()));
        }
    
        let mut sliced: Vec<String> = vec![ line[..4].to_string() ];
    
        if line.len() > 5 {
            let arg_slice = &line[5..];
            let mut t_begin: usize = 0;
            let mut mid_word: bool = false;
            let mut x = 0;
            while x < arg_slice.len() {
                let curr_char = arg_slice.chars().nth(x).unwrap();
                if curr_char == self.keyword_delimiter {
                    mid_word = !mid_word;
                    if !mid_word {
                        sliced.push(arg_slice[t_begin..(x + 1)].to_string());
                        t_begin = x + 1;
                        x += 1;
                    }
                }
                if curr_char == ' ' {
                    if !mid_word {
                        sliced.push(arg_slice[t_begin..x].to_string());
                        t_begin = x + 1;
                    }
                }
                x += 1;
            }
            if arg_slice.len() > t_begin {
                sliced.push(arg_slice[t_begin..arg_slice.len()].to_string());
            }
        }
        Ok(sliced)
    }

    fn parse_line(&self, split_line: Vec<String>) -> Result<(Instruction, Option<Vec<Arg>>), CompilerError> {
        // filter comments
        if split_line[0] == "note" { return Err(CompilerError::CommentParseError); }
        
        let instr: Instruction = split_line[0].parse()?;
        if split_line.len() == 1 { return Ok( (instr, None) ); }

        let signature = self.get_signature(&instr)?;

        let arg_slice = &split_line[1..];

        if signature.len() != arg_slice.len() {
            return Err(CompilerError::SignatureMismatch(instr, signature.len() as u8, arg_slice.len() as u8));
        }
        
        let mut args: Vec<Arg> = Vec::with_capacity(3);

        for x in 0..signature.len() {
            let mut err: Option<CompilerError> = None;
            let mut arg: Option<Arg> = None;
            for t in &signature[x] {
                match self.try_parse_arg(&arg_slice[x], &t) {
                    Ok(a) => {
                        arg = Some(a);
                        break;
                    },
                    Err(e) => {
                        match e {
                            CompilerError::NumberOutOfBounds => return Err(e),
                            _ => err = Some(e),
                        }
                    },
                }
            }
            if arg != None { args.push(arg.unwrap()); }
            else { return Err(err.unwrap()); }
        }
        Ok( (instr, Some(args)) )
    }

    fn get_signature(&self, instr: &Instruction) -> Result<Vec<Vec<ArgType>>, CompilerError> {
        match self.instruction_signatures.get(instr) {
            Some(s) => Ok(s.clone()),
            None => Err(CompilerError::InstructionNotAllowed(instr.to_owned())),
        }
    }

    fn try_parse_arg(&self, str: &String, at: &ArgType) -> Result<Arg, CompilerError> {
        match at {
            ArgType::Register => {
                if ("xtfm".contains(str) && str.len() == 1) || str.starts_with('#') {
                    return match &str[..] {
                        "x" => Ok(Arg::Register(RegLabel::X)),
                        "t" => Ok(Arg::Register(RegLabel::T)),
                        "f" => Ok(Arg::Register(RegLabel::F)),
                        "m" => Ok(Arg::Register(RegLabel::M)),
                        _ => Ok(Arg::Register(RegLabel::H(str.to_owned()))),
                    };
                }
                else { return Err(CompilerError::ArgTypeMismatch(str.to_owned(), at.to_owned())); }
            },
            ArgType::Number => match str.parse::<i16>() {
                Ok(n) => {
                    if n < -9999 || n > 9999 { return Err(CompilerError::NumberOutOfBounds); }
                    return Ok(Arg::Number(n));
                },
                Err(_) => return Err(CompilerError::ArgTypeMismatch(str.to_owned(), at.to_owned())),
            },
            ArgType::Comparison => {
                if self.comparisons.contains(str) {
                    let comp = match &str[..] {
                        "=" | "==" => Comp::Eq,
                        ">" => Comp::Gt,
                        "<" => Comp::Lt,
                        ">=" => Comp::Ge,
                        "<=" => Comp::Le,
                        "!=" => Comp::Ne,
                        _ => return Err(CompilerError::InvalidComparison(str.to_owned()))
                    };
                    return Ok(Arg::Comp(comp));
                }
                else { return Err(CompilerError::ArgTypeMismatch(str.to_owned(), at.to_owned())); }
            }
            ArgType::Keyword => {
                if str.starts_with(self.keyword_delimiter) && str.ends_with(self.keyword_delimiter) {
                    return Ok(Arg::Keyword(str.to_owned()));
                }
                else { return Err(CompilerError::ArgTypeMismatch(str.to_owned(), at.to_owned())); }
            },
            ArgType::Label => return Ok(Arg::Label(str.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    extra_instructions: bool,
    keyword_literals: bool,
    full_comparisons: bool,
    allow_multi_m: bool,
    keyword_delimiter: char,
    comment_prefixes: Vec<String>,
}

impl CompilerConfig {
    pub fn default() -> Self {
        Self::custom(false, false, false, false, '\'', vec![
            "note",
            ";;",
        ].into_iter().map(|s| s.to_string()).collect())
    }

    pub fn extended() -> Self {
        Self::custom(true, true, true, true, '\'', vec![
            "note",
            ";;",
            "//",
            "#",
        ].into_iter().map(|s| s.to_string()).collect())
    }

    pub fn custom(
        extra_instructions: bool,
        keyword_literals: bool,
        full_comparisons: bool,
        allow_multi_m: bool,
        keyword_delimiter: char,
        comment_prefixes: Vec<String>,
    ) -> Self {
        Self {
            extra_instructions,
            keyword_literals,
            full_comparisons,
            allow_multi_m,
            keyword_delimiter,
            comment_prefixes,
        }
    }

    pub fn generate_comparisons(&self) -> Vec<String> {
        match self.full_comparisons {
            true => vec!["=", ">", "<", ">=", "<=", "!="].into_iter().map(|s| s.to_string()).collect(),
            false => vec!["=", ">", "<"].into_iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn generate_signatures(&self) -> HashMap<Instruction, Vec<Vec<ArgType>>> {
        let r = vec![ArgType::Register];
        let rn = vec![ArgType::Register, ArgType::Number];
        let vari = match self.keyword_literals {
            true => vec![ArgType::Register, ArgType::Number, ArgType::Keyword],
            false => rn.clone(),
        };
        let c = vec![ArgType::Comparison];
        let l = vec![ArgType::Label];

        let mut sigs: HashMap<Instruction, Vec<Vec<ArgType>>> = HashMap::from_iter([
            (Instruction::Copy, vec![vari.clone(), r.clone()]),

            (Instruction::Addi, vec![rn.clone(), rn.clone(), r.clone()]),
            (Instruction::Subi, vec![rn.clone(), rn.clone(), r.clone()]),
            (Instruction::Muli, vec![rn.clone(), rn.clone(), r.clone()]),
            (Instruction::Divi, vec![rn.clone(), rn.clone(), r.clone()]),
            (Instruction::Modi, vec![rn.clone(), rn.clone(), r.clone()]),
            (Instruction::Swiz, vec![rn.clone(), rn.clone(), r.clone()]),

            (Instruction::Test, vec![vari.clone(), c.clone(), vari.clone()]),

            (Instruction::Mark, vec![l.clone()]),
            (Instruction::Jump, vec![l.clone()]),
            (Instruction::Fjmp, vec![l.clone()]),
            (Instruction::Tjmp, vec![l.clone()]),

            (Instruction::Repl, vec![l.clone()]),

            (Instruction::Link, vec![rn.clone()]),

            (Instruction::Noop, vec![]),
            (Instruction::Halt, vec![]),
            (Instruction::Kill, vec![]),
        ]);

        if self.extra_instructions {
            sigs.insert(Instruction::Prnt, vec![vari.clone()]);
        }

        sigs
    }
}
