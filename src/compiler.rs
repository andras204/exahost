use std::{collections::HashMap, fmt::Display};

use crate::{
    config::CompilerConfig,
    exa::{Arg, Comp, InstrTuple, Instruction, RegLabel},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgType {
    Number,
    Keyword,
    Register,
    Label,
    Comparison,
}

impl Display for ArgType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number => write!(f, "Number"),
            Self::Keyword => write!(f, "Keyword"),
            Self::Register => write!(f, "Register"),
            Self::Label => write!(f, "Label"),
            Self::Comparison => write!(f, "Comparison"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Compiler {
    instruction_signatures: HashMap<Instruction, Vec<Vec<ArgType>>>,
    comparisons: Vec<String>,
    // allow_multi_m: bool,
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
            // allow_multi_m: config.allow_multi_m,
            keyword_delimiter: config.keyword_delimiter,
            comment_prefixes: config.comment_prefixes,
        }
    }

    pub fn with_config(config: &CompilerConfig) -> Self {
        Self {
            instruction_signatures: config.generate_signatures(),
            comparisons: config.generate_comparisons(),
            // allow_multi_m: config.allow_multi_m,
            keyword_delimiter: config.keyword_delimiter,
            comment_prefixes: config.comment_prefixes.clone(),
        }
    }

    pub fn compile(&self, source: &[&str]) -> Result<Box<[InstrTuple]>, Vec<Error>> {
        if source.is_empty() {
            return Ok(vec![].into_boxed_slice());
        }
        let mut errs = Vec::new();
        let mut compiled: Vec<InstrTuple> = Vec::with_capacity(source.len());
        let source = match self.expand_macros(source) {
            Ok(s) => s,
            Err((s, mut e)) => {
                errs.append(&mut e);
                s
            }
        };
        for (x, line) in source.iter().enumerate() {
            if line.is_empty()
                || line.to_lowercase().starts_with("@rep")
                || line.to_lowercase().starts_with("@end")
            {
                continue;
            }

            let split = match self.split_line(line) {
                Ok(s) => s,
                Err(e) => {
                    errs.push(e.add_row(x));
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
                    errs.push(e.add_row(x));
                    continue;
                }
            };

            // check for multi M use
            if Self::check_multi_m(instr.arg_refs()) {
                errs.push(Error::new(x, ErrorKind::MultiMUse));
                continue;
            }
            compiled.push(instr);
        }

        match self.bake_labels(&mut compiled) {
            Ok(_) => (),
            Err(mut e) => errs.append(&mut e),
        }

        if !errs.is_empty() {
            Err(errs)
        } else {
            Ok(compiled.into_boxed_slice())
        }
    }

    fn expand_macros(&self, raw: &[&str]) -> Result<Vec<String>, (Vec<String>, Vec<Error>)> {
        let mut errs = Vec::new();
        let mut expanded = Vec::with_capacity(raw.len());
        let mut start = 0;
        let mut in_macro = false;
        for (x, line) in raw.iter().enumerate() {
            if line.is_empty() {
                expanded.push(String::new());
                continue;
            }
            // if @rep && not in macro -> we are in a macro -> append prev lines to raw
            // if @end && in macro -> macro over -> do the expansion
            // if @end && not in macro -> error: loose @end
            // if @rep && in macro -> error: no @end
            // remove correct macro tags
            // replace bad tags with empty line to preserve line numbers
            if line.to_lowercase().starts_with("@rep") {
                if in_macro {
                    errs.push(Error::new(x, ErrorKind::NestedMacros));
                    expanded.push(String::new());
                } else {
                    start = x;
                    in_macro = true;
                }
            } else if line.to_lowercase().starts_with("@end") {
                if in_macro {
                    in_macro = false;
                    let mac_slice = &raw[start..=x];
                    match self.expand_macro_slice(start, mac_slice) {
                        Ok(mut s) => expanded.append(&mut s),
                        Err((mut s, mut e)) => {
                            expanded.append(&mut s);
                            errs.append(&mut e);
                        }
                    };
                } else {
                    errs.push(Error::new(x, ErrorKind::NoStartTag));
                    expanded.push(String::new());
                }
            } else if !in_macro {
                expanded.push(line.to_string());
            }
        }
        if in_macro {
            errs.push(Error::new(start, ErrorKind::NoEndTag));
        }
        if !errs.is_empty() {
            Err((expanded, errs))
        } else {
            Ok(expanded)
        }
    }

    fn expand_macro_slice(
        &self,
        start: usize,
        slice: &[&str],
    ) -> Result<Vec<String>, (Vec<String>, Vec<Error>)> {
        let mut expanded = Vec::with_capacity(slice.len());
        let mut errs = Vec::new();

        // compile the macro slice to check for errors before expanding the macro
        // self.compile(&slice[1..(slice.len() - 1)]);

        match Self::repeat_macro(start, slice) {
            Ok(mut s) => expanded.append(&mut s),
            Err((mut s, mut e)) => {
                if s.is_empty() {
                    expanded.push(String::new());
                    expanded.append(
                        &mut slice[1..(slice.len() - 1)]
                            .iter()
                            .map(|l| l.to_string())
                            .collect(),
                    );
                    expanded.push(String::new());
                } else {
                    errs.append(&mut e);
                    expanded.push(String::new());
                    expanded.append(&mut s);
                    expanded.push(String::new());
                }
            }
        }
        if !errs.is_empty() {
            Err((expanded, errs))
        } else {
            Ok(expanded)
        }
    }

    fn repeat_macro(
        start: usize,
        original: &[&str],
    ) -> Result<Vec<String>, (Vec<String>, Vec<Error>)> {
        let macro_begin = original[0];
        let n = match match macro_begin.split(' ').nth(1) {
            Some(s) => s,
            None => {
                return Err((
                    vec![],
                    vec![Error::new(
                        start,
                        ErrorKind::InvalidMacroSyntax(macro_begin.to_string()),
                    )],
                ))
            }
        }
        .parse::<usize>()
        {
            Ok(n) => n,
            Err(_) => {
                return Err((
                    vec![],
                    vec![Error::new(
                        start,
                        ErrorKind::InvalidMacroSyntax(macro_begin.to_string()),
                    )],
                ))
            }
        };

        let mut errs = Vec::new();
        let mut expanded = Vec::with_capacity((original.len() - 2) * n);

        for x in 1..n {
            let mut int_errs = Vec::new();
            for (y, line) in original.iter().enumerate() {
                if line.is_empty() {
                    continue;
                }
                if line.to_lowercase().starts_with("@end") {
                    break;
                }
                match Self::substitute_macro(line, x) {
                    Ok(l) => expanded.push(l),
                    Err(e) => {
                        int_errs.push(e.add_row(start + y));
                        expanded.push(String::new());
                    }
                }
            }
            if !int_errs.is_empty() {
                errs.append(&mut int_errs);
                break;
            }
        }
        if !errs.is_empty() {
            Err((expanded, errs))
        } else {
            Ok(expanded)
        }
    }

    fn substitute_macro(line: &str, n: usize) -> Result<String, Error> {
        let mut line = line.to_string();
        while line.contains("@{") {
            let start = line
                .chars()
                .position(|c| c == '{')
                .ok_or(Error::new_no_pos(ErrorKind::InvalidMacroSyntax(
                    line.to_string(),
                )))?
                - 1;
            let end = line
                .chars()
                .position(|c| c == '}')
                .ok_or(Error::new_no_pos(ErrorKind::InvalidMacroSyntax(
                    line.to_string(),
                )))?;

            let s = line.drain(start..=end).collect::<String>();
            let split = s.split(',').collect::<Vec<&str>>();
            let x = split[0]
                .trim_start_matches("@{")
                .parse::<i16>()
                .map_err(|_| Error::new_no_pos(ErrorKind::InvalidMacroSyntax(line.to_string())))?;
            let y = split[1]
                .trim_end_matches('}')
                .parse::<i16>()
                .map_err(|_| Error::new_no_pos(ErrorKind::InvalidMacroSyntax(line.to_string())))?;
            line.insert_str(start, &(x + (y * n as i16)).to_string()[..]);
        }
        Ok(line)
    }

    fn split_line(&self, line: &str) -> Result<Vec<String>, Error> {
        // unify comments
        for c in &self.comment_prefixes {
            if line.to_lowercase().starts_with(c) {
                return Ok(vec!["note".to_string()]);
            }
        }

        // filter out short lines
        if line.len() < 4 {
            return Err(Error::new_no_pos(ErrorKind::UnknownInstruction(
                line.to_string(),
            )));
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
                sliced.push(arg_slice[t_begin..].to_string());
            }
        }
        Ok(sliced)
    }

    fn parse_line(&self, split_line: Vec<String>) -> Result<InstrTuple, Error> {
        let instr: Instruction = match split_line[0].parse() {
            Ok(i) => i,
            Err(_) => {
                return Err(Error::new_no_pos(ErrorKind::UnknownInstruction(
                    split_line[0].to_string(),
                )))
            }
        };

        if instr == Instruction::Test && split_line.len() == 2 {
            match &split_line[1].to_lowercase()[..] {
                "mrd" => return Ok(InstrTuple(Instruction::TestMrd, None, None, None)),
                "eof" => return Ok(InstrTuple(Instruction::TestEof, None, None, None)),
                _ => (),
            }
        }

        let signature = self.get_signature(&instr)?;

        let arg_slice = &split_line[1..];

        if signature.len() != arg_slice.len() {
            return Err(Error::new_no_pos(ErrorKind::SignatureMismatch(
                instr,
                signature.len() as u8,
                arg_slice.len() as u8,
            )));
        }

        let mut args: Vec<Arg> = Vec::with_capacity(3);

        for x in 0..signature.len() {
            let mut err: Option<Error> = None;
            let mut arg: Option<Arg> = None;
            for t in &signature[x] {
                match self.try_parse_arg(&arg_slice[x], t) {
                    Ok(a) => {
                        arg = Some(a);
                        break;
                    }
                    Err(e) => match e.kind {
                        ErrorKind::NumberOutOfBounds(_) => return Err(e),
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

        Ok(InstrTuple(instr, args.0, args.1, args.2))
    }

    fn unpack_arg_vec(args: Vec<Arg>) -> (Option<Arg>, Option<Arg>, Option<Arg>) {
        let arg0 = args.first().map(|a| a.to_owned());
        let arg1 = args.get(1).map(|a| a.to_owned());
        let arg2 = args.get(2).map(|a| a.to_owned());
        (arg0, arg1, arg2)
    }

    fn get_signature(&self, instr: &Instruction) -> Result<Vec<Vec<ArgType>>, Error> {
        match self.instruction_signatures.get(instr) {
            Some(s) => Ok(s.to_owned()),
            None => Err(Error::new_no_pos(ErrorKind::InstructionNotAllowed(*instr))),
        }
    }

    #[cfg(not(feature = "full-register-range"))]
    fn try_parse_arg(&self, str: &String, at: &ArgType) -> Result<Arg, Error> {
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
                    Err(Error::new_no_pos(ErrorKind::ArgTypeMismatch(
                        at.to_owned(),
                        str.to_owned(),
                    )))
                }
            }
            ArgType::Number => match str.parse::<i16>() {
                Ok(n) => {
                    if !(-9999..=9999).contains(&n) {
                        return Err(Error::new_no_pos(ErrorKind::NumberOutOfBounds(n)));
                    }
                    Ok(Arg::Number(n))
                }
                Err(_) => Err(Error::new_no_pos(ErrorKind::ArgTypeMismatch(
                    at.to_owned(),
                    str.to_owned(),
                ))),
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
                        _ => {
                            return Err(Error::new_no_pos(ErrorKind::InvalidComparison(
                                str.to_owned(),
                            )))
                        }
                    };
                    Ok(Arg::Comp(comp))
                } else {
                    Err(Error::new_no_pos(ErrorKind::ArgTypeMismatch(
                        at.to_owned(),
                        str.to_owned(),
                    )))
                }
            }
            ArgType::Keyword => {
                if str.starts_with(self.keyword_delimiter) && str.ends_with(self.keyword_delimiter)
                {
                    Ok(Arg::Keyword(str.to_owned().into_boxed_str()))
                } else {
                    Err(Error::new_no_pos(ErrorKind::ArgTypeMismatch(
                        at.to_owned(),
                        str.to_owned(),
                    )))
                }
            }
            ArgType::Label => Ok(Arg::JumpLabel(str.to_owned().into_boxed_str())),
        }
    }

    #[cfg(feature = "full-register-range")]
    fn try_parse_arg(&self, str: &String, at: &ArgType) -> Result<Arg, Error> {
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
                    Err(Error::new_no_pos(ErrorKind::ArgTypeMismatch(
                        at.to_owned(),
                        str.to_owned(),
                    )))
                }
            }
            ArgType::Number => match str.parse::<i16>() {
                Ok(n) => Ok(Arg::Number(n)),
                Err(_) => Err(Error::new_no_pos(ErrorKind::ArgTypeMismatch(
                    at.to_owned(),
                    str.to_owned(),
                ))),
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
                        _ => {
                            return Err(Error::new_no_pos(ErrorKind::InvalidComparison(
                                str.to_owned(),
                            )))
                        }
                    };
                    Ok(Arg::Comp(comp))
                } else {
                    Err(Error::new_no_pos(ErrorKind::ArgTypeMismatch(
                        at.to_owned(),
                        str.to_owned(),
                    )))
                }
            }
            ArgType::Keyword => {
                if str.starts_with(self.keyword_delimiter) && str.ends_with(self.keyword_delimiter)
                {
                    Ok(Arg::Keyword(str.to_owned().into_boxed_str()))
                } else {
                    Err(Error::new_no_pos(ErrorKind::ArgTypeMismatch(
                        at.to_owned(),
                        str.to_owned(),
                    )))
                }
            }
            ArgType::Label => Ok(Arg::JumpLabel(str.to_owned().into_boxed_str())),
        }
    }

    fn check_multi_m(args: (Option<&Arg>, Option<&Arg>, Option<&Arg>)) -> bool {
        let mut m = 0;
        if args.0.is_some() && args.0.unwrap().is_reg_m() {
            m += 1;
        }
        if args.1.is_some() && args.1.unwrap().is_reg_m() {
            m += 1;
        }
        if args.2.is_some() && args.2.unwrap().is_reg_m() {
            m += 1;
        }
        m > 1
    }

    fn bake_labels(&self, instrs: &mut Vec<InstrTuple>) -> Result<(), Vec<Error>> {
        let mut label_map = HashMap::new();
        let mut errs = Vec::new();
        let mut len = instrs.len();
        let mut x = 0;
        while x < len {
            if let Instruction::Mark = instrs[x].0 {
                let i = instrs.remove(x);
                label_map.insert(i.one_arg(), x);
                len -= 1;
            }
            x += 1;
        }
        for x in 0..instrs.len() {
            match instrs[x].0 {
                Instruction::Jump | Instruction::Fjmp | Instruction::Tjmp | Instruction::Repl => {
                    let label = instrs[x].clone().one_arg();
                    match label_map.get(&label) {
                        Some(n) => {
                            // replace original label arg with index (n)
                            // might require creating a separate Vec
                            // that would also simplify error indexing
                            instrs[x].1 = Some(Arg::Number(*n as i16));
                        }
                        None => {
                            errs.push(Error::new(x, ErrorKind::UndefinedLabel(label.to_string())))
                        }
                    }
                }
                _ => continue,
            }
        }
        if !errs.is_empty() {
            Err(errs)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub row: Option<usize>,
    pub kind: ErrorKind,
}

impl Error {
    pub fn new(row: usize, kind: ErrorKind) -> Self {
        Self {
            row: Some(row),
            kind,
        }
    }

    pub fn new_no_pos(kind: ErrorKind) -> Self {
        Self { row: None, kind }
    }

    pub fn add_row(mut self, row: usize) -> Self {
        self.row = Some(row);
        self
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let row = if let Some(r) = self.row.as_ref() {
            r.to_string()
        } else {
            "?".to_string()
        };
        write!(f, "Error on line {}: {}", row, self.kind)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    UnknownInstruction(String),
    InstructionNotAllowed(Instruction),
    CommentParseError,
    ArgTypeMismatch(ArgType, String),
    NumberOutOfBounds(i16),
    SignatureMismatch(Instruction, u8, u8),
    InvalidComparison(String),
    NestedMacros,
    NoEndTag,
    NoStartTag,
    InvalidMacroSyntax(String),
    MultiMUse,
    UndefinedLabel(String),
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_msg = match self {
            Self::UnknownInstruction(s) => format!("unknown instruction: '{}'", s),
            Self::InstructionNotAllowed(i) => {
                format!("disallowed instruction: {}", i)
            }
            Self::CommentParseError => "comments cannot be parsed".to_string(),
            Self::ArgTypeMismatch(t, s) => format!("cannot parse '{}' as {}", s, t),
            Self::NumberOutOfBounds(n) => format!("{} is out of range -9999..=9999", n),
            Self::SignatureMismatch(i, s, g) => {
                format!("{} requires {} arguments, found {}", i, s, g)
            }
            Self::InvalidComparison(s) => format!("invalid comparison operator '{}'", s),
            Self::NestedMacros => "macros cannot be nested".to_string(),
            Self::NoEndTag => "@REP macro missing @END tag".to_string(),
            Self::NoStartTag => "@END tag without starting macro".to_string(),
            Self::InvalidMacroSyntax(s) => format!("invalid macro syntax: '{}'", s),
            Self::MultiMUse => "multiple use of M register".to_string(),
            Self::UndefinedLabel(l) => format!("undefined label '{}'", l),
        };
        write!(f, "{}", err_msg)
    }
}
