use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::exa::{Arg, Comp, InstrTuple, Instruction, RegLabel};

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
            Self::InstructionNotAllowed(i) => {
                format!("use of {:?} disallowed by compiler configuration", i)
            }
            Self::CommentParseError => "comments cannot be parsed".to_string(),
            Self::ArgTypeMismatch(s, t) => format!("cannot convert '{}' to {:?}", s, t),
            Self::NumberOutOfBounds => "number too large or too small".to_string(),
            Self::SignatureMismatch(i, s, g) => {
                format!("{:?} requires {} arguments, found {}", i, s, g)
            }
            Self::InvalidComparison(s) => format!("invalid comparison operator '{}'", s),
        };
        write!(f, "{}", err_msg)
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

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn compile(&self, source: Vec<String>) -> Result<Box<[InstrTuple]>, Vec<String>> {
        let mut errs: Vec<String> = Vec::new();
        let mut compiled: Vec<InstrTuple> = Vec::with_capacity(source.len());
        if source.is_empty() {
            errs.push("nothing to compile".to_string());
        }
        for (x, line) in source.iter().enumerate() {
            let split = match self.split_line(line) {
                Ok(s) => s,
                Err(e) => {
                    errs.push(format!("Error on line {}: {}", x, e));
                    continue;
                }
            };
            // filter comments
            if split[0] == "note" {
                continue;
            }
            let instr = match self.parse_line(split) {
                Ok(i) => i,
                Err(e) => {
                    errs.push(format!("Error on line {}: {}", x, e));
                    continue;
                }
            };
            // check for multi M use
            if !self.allow_multi_m && Self::check_multi_m((&instr.1, &instr.2, &instr.3)) {
                errs.push(format!("Error on line {}: multiple M use not allowed", x));
                continue;
            }
            compiled.push(instr);
        }
        if !errs.is_empty() {
            return Err(errs);
        }
        Ok(compiled.into_boxed_slice())
    }

    fn check_multi_m(args: (&Option<Arg>, &Option<Arg>, &Option<Arg>)) -> bool {
        let mut m = 0;
        if args.0.is_some() && args.0.as_ref().unwrap().is_reg_m() {
            m += 1;
        }
        if args.1.is_some() && args.1.as_ref().unwrap().is_reg_m() {
            m += 1;
        }
        if args.2.is_some() && args.2.as_ref().unwrap().is_reg_m() {
            m += 1;
        }
        m > 1
    }

    fn split_line(&self, line: &str) -> Result<Vec<String>, CompilerError> {
        // unify comments
        for c in &self.comment_prefixes {
            if line.starts_with(c) {
                return Ok(vec!["note".to_string()]);
            }
        }

        // filter out short lines
        if line.len() < 4 {
            return Err(CompilerError::UnknownInstruction(line.to_owned()));
        }

        let mut sliced: Vec<String> = vec![line[..4].to_string()];

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
                        x += 1;
                        t_begin = x + 1;
                    }
                }
                if curr_char == ' ' && !mid_word {
                    sliced.push(arg_slice[t_begin..x].to_string());
                    t_begin = x + 1;
                }
                x += 1;
            }
            if arg_slice.len() > t_begin {
                sliced.push(arg_slice[t_begin..arg_slice.len()].to_string());
            }
        }
        Ok(sliced)
    }

    fn parse_line(&self, split_line: Vec<String>) -> Result<InstrTuple, CompilerError> {
        let instr: Instruction = match split_line[0].parse() {
            Ok(i) => i,
            Err(_) => return Err(CompilerError::UnknownInstruction(split_line[0].clone())),
        };
        if split_line.len() == 1 {
            return Ok((instr, None, None, None));
        }

        let signature = self.get_signature(&instr)?;

        let arg_slice = &split_line[1..];

        if signature.len() != arg_slice.len() {
            return Err(CompilerError::SignatureMismatch(
                instr,
                signature.len() as u8,
                arg_slice.len() as u8,
            ));
        }

        let mut args: Vec<Arg> = Vec::with_capacity(3);

        for x in 0..signature.len() {
            let mut err: Option<CompilerError> = None;
            let mut arg: Option<Arg> = None;
            for t in &signature[x] {
                match self.try_parse_arg(&arg_slice[x], t) {
                    Ok(a) => {
                        arg = Some(a);
                        break;
                    }
                    Err(e) => match e {
                        CompilerError::NumberOutOfBounds => return Err(e),
                        _ => err = Some(e),
                    },
                }
            }
            if let Some(a) = arg {
                args.push(a);
            } else {
                return Err(err.unwrap());
            }
        }

        let args = Self::unpack_arg_vec(args);

        Ok((instr, args.0, args.1, args.2))
    }

    fn unpack_arg_vec(args: Vec<Arg>) -> (Option<Arg>, Option<Arg>, Option<Arg>) {
        let arg0 = args.first().map(|a| a.to_owned());
        let arg1 = args.get(1).map(|a| a.to_owned());
        let arg2 = args.get(2).map(|a| a.to_owned());
        (arg0, arg1, arg2)
    }

    fn get_signature(&self, instr: &Instruction) -> Result<Vec<Vec<ArgType>>, CompilerError> {
        match self.instruction_signatures.get(instr) {
            Some(s) => Ok(s.to_owned()),
            None => Err(CompilerError::InstructionNotAllowed(instr.to_owned())),
        }
    }

    fn try_parse_arg(&self, str: &String, at: &ArgType) -> Result<Arg, CompilerError> {
        match at {
            ArgType::Register => {
                if ("xtfm".contains(str) && str.len() == 1) || str.starts_with('#') {
                    match &str[..] {
                        "x" => Ok(Arg::RegLabel(RegLabel::X)),
                        "t" => Ok(Arg::RegLabel(RegLabel::T)),
                        "f" => Ok(Arg::RegLabel(RegLabel::F)),
                        "m" => Ok(Arg::RegLabel(RegLabel::M)),
                        _ => Ok(Arg::RegLabel(RegLabel::H(str.to_owned()))),
                    }
                } else {
                    Err(CompilerError::ArgTypeMismatch(
                        str.to_owned(),
                        at.to_owned(),
                    ))
                }
            }
            ArgType::Number => match str.parse::<i16>() {
                Ok(n) => {
                    if !(-9999..=9999).contains(&n) {
                        return Err(CompilerError::NumberOutOfBounds);
                    }
                    Ok(Arg::Number(n))
                }
                Err(_) => Err(CompilerError::ArgTypeMismatch(
                    str.to_owned(),
                    at.to_owned(),
                )),
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
                        _ => return Err(CompilerError::InvalidComparison(str.to_owned())),
                    };
                    Ok(Arg::Comp(comp))
                } else {
                    Err(CompilerError::ArgTypeMismatch(
                        str.to_owned(),
                        at.to_owned(),
                    ))
                }
            }
            ArgType::Keyword => {
                if str.starts_with(self.keyword_delimiter) && str.ends_with(self.keyword_delimiter)
                {
                    Ok(Arg::Keyword(str.to_owned()))
                } else {
                    Err(CompilerError::ArgTypeMismatch(
                        str.to_owned(),
                        at.to_owned(),
                    ))
                }
            }
            ArgType::Label => Ok(Arg::JumpLabel(str.to_owned())),
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

impl Default for CompilerConfig {
    fn default() -> Self {
        Self::custom(
            false,
            false,
            false,
            false,
            '\'',
            vec!["note", ";;"]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        )
    }
}

impl CompilerConfig {
    pub fn extended() -> Self {
        Self::custom(
            true,
            true,
            true,
            true,
            '\'',
            vec!["note", ";;", "//", "#"]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        )
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
            true => vec!["=", ">", "<", ">=", "<=", "!="]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            false => vec!["=", ">", "<"]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
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
            (Instruction::Rand, vec![rn.clone(), rn.clone(), r.clone()]),
            (
                Instruction::Test,
                vec![vari.clone(), c.clone(), vari.clone()],
            ),
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
