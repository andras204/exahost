use regex::{Match, Regex};

use crate::exa::{Arg, Instruction, OpCode};
use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Deref, DerefMut, Range},
};

pub use self::config::Config;

mod config;

#[derive(Debug, Clone)]
pub struct Token {
    pub row: usize,
    pub col: usize,
    pub content: String,
    pub ttype: TokenType,
}

impl Token {
    fn new<T>(row: usize, col: usize, content: T, ttype: TokenType) -> Self
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

#[derive(Debug, Clone)]
pub struct Error {
    pub row: usize,
    pub col: usize,
    pub context: String,
    pub etype: ErrorType,
}

impl Error {
    fn new<T>(row: usize, col: usize, content: T, etype: ErrorType) -> Self
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

    fn from_token(t: Token, etype: ErrorType) -> Self {
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

#[derive(Debug, Clone)]
struct Signature(pub Vec<Vec<TokenType>>);

impl Signature {
    pub fn one(ttypes: &[TokenType]) -> Self {
        Self(vec![ttypes.to_owned(), vec![], vec![]])
    }

    pub fn two(ttypes1: &[TokenType], ttypes2: &[TokenType]) -> Self {
        Self(vec![ttypes1.to_owned(), ttypes2.to_owned(), vec![]])
    }

    pub fn three(ttypes1: &[TokenType], ttypes2: &[TokenType], ttypes3: &[TokenType]) -> Self {
        Self(vec![
            ttypes1.to_owned(),
            ttypes2.to_owned(),
            ttypes3.to_owned(),
        ])
    }

    pub fn label() -> Self {
        Self(vec![vec![TokenType::JumpLabel], vec![], vec![]])
    }

    pub fn r() -> Self {
        Self(vec![vec![TokenType::RegisterLabel], vec![], vec![]])
    }

    pub fn rn() -> Self {
        Self(vec![
            vec![TokenType::RegisterLabel, TokenType::Number],
            vec![],
            vec![],
        ])
    }

    pub fn math() -> Self {
        Self(vec![
            vec![TokenType::Number, TokenType::RegisterLabel],
            vec![TokenType::Number, TokenType::RegisterLabel],
            vec![TokenType::RegisterLabel],
        ])
    }

    pub fn empty() -> Self {
        Self(vec![vec![], vec![], vec![]])
    }

    pub fn len(&self) -> usize {
        let mut len = 0;
        if !self.0[0].is_empty() {
            len += 1;
        }
        if !self.0[1].is_empty() {
            len += 1;
        }
        if !self.0[2].is_empty() {
            len += 1;
        }
        len
    }
}

#[derive(Debug, Clone)]
pub struct Compiler {
    instruction_signatures: HashMap<OpCode, Signature>,
    comparisons: Vec<String>,
    keyword_delimiter: char,
    comment_prefixes: Vec<String>,
    macro_regex: Regex,
}

impl Compiler {
    pub fn new(config: Config) -> Self {
        Self {
            instruction_signatures: config.generate_signatures(),
            comparisons: config.generate_comparisons(),
            keyword_delimiter: config.keyword_delimiter,
            comment_prefixes: config.comment_prefixes,
            // disallow space in macro
            //macro_regex: Regex::new(r"@\{-?\d{1,4}, ?-?\d{1,4}\}").unwrap(),
            macro_regex: Regex::new(r"@\{-?\d{1,4},-?\d{1,4}\}").unwrap(),
        }
    }

    pub fn compile(&self, raw: &[&str]) -> Result<Box<[Instruction]>, Vec<Error>> {
        let mut tokens = self.tokenize(raw);
        tokens = self.expand_macros(tokens);
        self.typecheck(&mut tokens);
        Self::bake_jumps(&mut tokens);
        let errs = Self::extract_errs(&tokens);
        // convert to instruction
        if !errs.is_empty() {
            return Err(errs);
        }
        Ok(self.lines_to_instructions(tokens))
    }

    fn lines_to_instructions(&self, lines: Vec<Line>) -> Box<[Instruction]> {
        let mut instrs = Vec::with_capacity(lines.len());
        for line in lines.into_iter() {
            let op: OpCode = line[0].clone().unwrap().try_into().unwrap();
            let args: Vec<Arg> = line[1..]
                .into_iter()
                .map(|t| self.token_to_arg(t.clone().unwrap()).unwrap())
                .collect();
            instrs.push(Instruction(
                op,
                match args.get(0) {
                    Some(a) => Some(a.clone()),
                    None => None,
                },
                match args.get(1) {
                    Some(a) => Some(a.clone()),
                    None => None,
                },
                match args.get(2) {
                    Some(a) => Some(a.clone()),
                    None => None,
                },
            ));
        }
        instrs.into_boxed_slice()
    }

    fn token_to_arg(&self, t: Token) -> Result<Arg, Error> {
        match t.ttype {
            TokenType::Number => Ok(Arg::Number(t.content.parse().unwrap())),
            TokenType::Keyword => Ok(Arg::Keyword(
                t.content.trim_matches(self.keyword_delimiter).into(),
            )),
            TokenType::JumpLabel => Ok(Arg::JumpIndex(t.content.parse().unwrap())),
            TokenType::Comparison => Ok(Arg::Comp(t.content.parse().unwrap())),
            TokenType::RegisterLabel => Ok(Arg::RegLabel(t.content.parse().unwrap())),
            TokenType::OpCode
            | TokenType::Comment
            | TokenType::MacroEnd
            | TokenType::MacroStart
            | TokenType::MacroReplace => Err(Error::from_token(t, ErrorType::NotArg)),
        }
    }

    fn extract_errs(lines: &[Line]) -> Vec<Error> {
        let mut errs = Vec::new();
        for line in lines {
            if !line.has_error() {
                continue;
            }
            for res in line.iter() {
                match res {
                    Ok(_) => (),
                    Err(e) => errs.push(e.clone()),
                }
            }
        }
        errs
    }

    fn bake_jumps(lines: &mut Vec<Line>) {
        let mut label_map = HashMap::new();
        let mut len = lines.len();
        let mut x = 0;
        while x < len {
            if lines[x].has_error() {
                x += 1;
                continue;
            }
            if let Ok(op) = lines[x][0].clone().unwrap().try_into() {
                match op {
                    OpCode::Mark => {
                        let label = lines[x][1].clone().unwrap();
                        if label.ttype == TokenType::JumpLabel {
                            if label_map.contains_key(&label.content) {
                                lines[x]
                                    .push(Err(Error::from_token(label, ErrorType::DuplicateLabel)));
                                x += 1;
                                continue;
                            }
                            lines.remove(x);
                            label_map.insert(label.content, x);
                            len -= 1;
                        }
                    }
                    _ => {
                        x += 1;
                        continue;
                    }
                }
            }
        }
        for line in lines.iter_mut() {
            if line.has_error() {
                continue;
            }
            if let Ok(op) = line[0].clone().unwrap().try_into() {
                match op {
                    OpCode::Jump | OpCode::Fjmp | OpCode::Tjmp | OpCode::Repl => {
                        let label = line[1].clone().unwrap();
                        if label.ttype == TokenType::JumpLabel {
                            if !label_map.contains_key(&label.content) {
                                line.push(Err(Error::from_token(label, ErrorType::UndefinedLabel)));
                                continue;
                            }
                            line[1] = Ok(Token::new(
                                label.row,
                                label.col,
                                label_map.get(&label.content).unwrap().to_string(),
                                TokenType::JumpLabel,
                            ));
                        }
                    }
                    _ => continue,
                }
            }
        }
    }

    fn typecheck(&self, lines: &mut [Line]) {
        for line in lines.iter_mut() {
            if line[0].is_err() {
                continue;
            }
            let op: OpCode = match line[0].clone().unwrap().try_into() {
                Ok(o) => o,
                Err(e) => {
                    line[0] = Err(e);
                    continue;
                }
            };
            let sig = match self.instruction_signatures.get(&op) {
                Some(sig) => sig,
                None => {
                    line[0] = Err(Error::from_token(
                        line[0].clone().unwrap(),
                        ErrorType::UnknownInstruction,
                    ));
                    continue;
                }
            };
            {
                let arg_slice = &mut line[1..];
                for x in 0..usize::min(arg_slice.len(), sig.len()) {
                    if arg_slice[x].is_err() {
                        continue;
                    }
                    if !sig.0[x].contains(&arg_slice[x].as_ref().unwrap().ttype) {
                        arg_slice[x] = Err(Error::from_token(
                            arg_slice[x].clone().unwrap(),
                            ErrorType::ArgTypeMismatch,
                        ))
                    }
                }
            }
            if line.len() - 1 != sig.len() {
                let row = line.row();
                let col = line.last_col();
                let len = line.len() - 1;
                line.push(Err(Error::new(
                    row,
                    col,
                    format!("Expected {} args, found {}", sig.len(), len),
                    ErrorType::SigLenMismatch,
                )));
            }
        }
    }

    fn expand_macros(&self, raw: Vec<Line>) -> Vec<Line> {
        let mut expanded = Vec::new();
        let mut in_macro = false;
        let mut start = 0;
        for (x, line) in raw.iter().enumerate() {
            let first = match line[0].as_ref() {
                Ok(t) => t,
                Err(_) => continue,
            };
            match first.ttype {
                TokenType::MacroStart => {
                    if in_macro {
                        expanded.push(
                            Error::from_token(first.to_owned(), ErrorType::NestedMacros).into(),
                        );
                    } else {
                        start = x;
                        in_macro = true;
                    }
                }
                TokenType::MacroEnd => {
                    if in_macro {
                        expanded.append(&mut self.repeat_macro(&raw[start..=x]));
                        in_macro = false;
                    } else {
                        expanded.push(
                            Error::from_token(first.to_owned(), ErrorType::MissingRepTag).into(),
                        );
                    }
                }
                _ => {
                    if !in_macro {
                        expanded.push(line.to_owned());
                    }
                }
            }
        }
        if in_macro {
            expanded.push(Error::new(start, 0, "", ErrorType::MissingEndTag).into());
        }
        expanded
    }

    fn repeat_macro(&self, lines: &[Line]) -> Vec<Line> {
        let rep_count = match Self::get_rep_count(&lines[0]) {
            Ok(r) => r,
            Err(e) => return vec![e.into()],
        };
        let mut expanded = Vec::with_capacity((lines.len() - 2) * rep_count);
        for x in 0..rep_count {
            for line in &lines[1..(lines.len() - 1)] {
                expanded.push(self.substitute_macro(line, x as i16));
            }
        }
        expanded
    }

    fn substitute_macro(&self, line: &Line, x: i16) -> Line {
        let mut new_line = Line::with_capacity(line.len());
        for res in line.iter() {
            match res {
                Ok(t) => {
                    if t.ttype == TokenType::MacroReplace {
                        let mut content = t.content.clone();
                        while let Some(r_match) = self.macro_regex.find(&content) {
                            let (range, base, inc) = Self::get_substitution_params(r_match);
                            let num = base + (x * inc);
                            content.replace_range(range, &num.to_string());
                        }
                        new_line.push(self.infer_arg_type(t.row, t.col, &content));
                    } else {
                        new_line.push(Ok(t.clone()));
                    }
                }
                Err(e) => new_line.push(Err(e.clone())),
            }
        }
        new_line
    }

    fn get_substitution_params(r_match: Match) -> (Range<usize>, i16, i16) {
        let range = r_match.range();
        let mut split = r_match.as_str().split(',');
        let base = split
            .next()
            .unwrap()
            .trim_start_matches("@{")
            .parse::<i16>()
            .unwrap();
        let inc = split
            .next()
            .unwrap()
            .trim_end_matches('}')
            .parse::<i16>()
            .unwrap();
        (range, base, inc)
    }

    fn get_rep_count(line: &Line) -> Result<usize, Error> {
        let arg = match line.get(1) {
            Some(r) => r,
            None => {
                return Err(Error::new(line.row(), 5, "", ErrorType::InvalidNumber));
            }
        };
        match arg {
            Ok(t) => {
                if t.ttype == TokenType::Number {
                    Ok(t.content.parse().unwrap())
                } else {
                    Err(Error::from_token(t.clone(), ErrorType::InvalidNumber))
                }
            }
            Err(e) => Err(e.clone()),
        }
    }

    fn tokenize(&self, raw: &[&str]) -> Vec<Line> {
        let mut tokenized: Vec<Line> = Vec::with_capacity(raw.len());
        for (x, line) in raw.iter().enumerate() {
            if line.is_empty() {
                continue;
            }

            // skip comments
            for c in self.comment_prefixes.iter() {
                if line.to_lowercase().starts_with(c) {
                    tokenized.push(Token::new(x, 0, line, TokenType::Comment).into());
                    continue;
                }
            }

            // filter out short lines
            if line.len() < 4 {
                tokenized.push(Error::new(x, 0, line, ErrorType::UnknownInstruction).into());
                continue;
            }

            let mut line_vec = Vec::with_capacity(4);
            let sliced = self.slice_line(line);

            // handle special tests
            if sliced[0].1.to_lowercase() == "test"
                && (sliced[1].1.to_lowercase() == "eof" || sliced[1].1.to_lowercase() == "mrd")
            {
                tokenized.push(
                    Token::new(x, 0, format!("test {}", sliced[1].1), TokenType::OpCode).into(),
                );
                continue;
            }

            let ttype = if sliced[0].1.to_lowercase() == "@rep" {
                TokenType::MacroStart
            } else if sliced[0].1.to_lowercase() == "@end" {
                TokenType::MacroEnd
            } else {
                TokenType::OpCode
            };

            line_vec.push(Ok(Token::new(x, 0, &sliced[0].1, ttype)));

            for (col, arg) in &sliced[1..] {
                line_vec.push(self.infer_arg_type(x, *col, arg));
            }

            tokenized.push(line_vec.into());
        }
        tokenized
    }

    fn infer_arg_type(&self, row: usize, col: usize, content: &String) -> Result<Token, Error> {
        let ttype = {
            if is_macro(&self.macro_regex, content) {
                TokenType::MacroReplace
            } else if is_keyword(self.keyword_delimiter, content) {
                TokenType::Keyword
            } else if is_number(content) {
                TokenType::Number
            } else if is_reg_label(content) {
                TokenType::RegisterLabel
            } else if is_comparison(&self.comparisons, content) {
                TokenType::Comparison
            } else {
                TokenType::JumpLabel
            }
        };
        return Ok(Token::new(row, col, content, ttype));

        #[inline(always)]
        fn is_number(arg: &str) -> bool {
            arg.parse::<i16>().is_ok()
        }

        #[inline(always)]
        fn is_macro(regex: &Regex, arg: &str) -> bool {
            regex.is_match(arg)
        }

        #[inline(always)]
        fn is_reg_label(arg: &str) -> bool {
            (arg.len() == 1 && "XTFM".contains(&arg.to_uppercase())) || arg.starts_with('#')
        }

        #[inline(always)]
        fn is_keyword(delim: char, arg: &str) -> bool {
            arg.starts_with(delim) && arg.ends_with(delim)
        }

        #[inline(always)]
        fn is_comparison(comparisons: &[String], arg: &String) -> bool {
            comparisons.contains(arg)
        }
    }

    fn slice_line(&self, line: &str) -> Vec<(usize, String)> {
        let mut sliced = vec![(0, line[..4].to_string())];

        if line.len() > 5 {
            let arg_slice = &line[5..];
            let mut start: usize = 0;
            let mut mid_word: bool = false;
            let mut x = 0;
            while x < arg_slice.len() {
                let curr_char = arg_slice.chars().nth(x).unwrap();
                if curr_char == self.keyword_delimiter {
                    mid_word = !mid_word;
                    if !mid_word {
                        sliced.push((start + 5, arg_slice[start..(x + 1)].to_string()));
                        x += 1;
                        start = x + 1;
                    }
                }
                if curr_char == ' ' && !mid_word {
                    sliced.push((start + 5, arg_slice[start..x].to_string()));
                    start = x + 1;
                }
                x += 1;
            }
            if arg_slice.len() > start {
                sliced.push((start + 5, arg_slice[start..].to_string()));
            }
        }
        sliced
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

#[derive(Debug, Clone)]
pub struct Line {
    inner: Vec<Result<Token, Error>>,
}

impl Line {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    pub fn has_error(&self) -> bool {
        for t in self.inner.iter() {
            if t.is_err() {
                return true;
            }
        }
        false
    }

    pub fn row(&self) -> usize {
        match &self[0] {
            Ok(t) => t.row,
            Err(e) => e.row,
        }
    }

    pub fn last_col(&self) -> usize {
        match self.inner.last().unwrap() {
            Ok(t) => t.col + t.content.len(),
            Err(e) => e.col + e.context.len(),
        }
    }
}

impl TryFrom<Token> for OpCode {
    type Error = Error;
    fn try_from(value: Token) -> Result<Self, Self::Error> {
        if value.ttype != TokenType::OpCode {
            return Err(Error::from_token(value, ErrorType::NotOpCode));
        }
        match value.content.parse::<OpCode>() {
            Ok(o) => Ok(o),
            Err(_) => Err(Error::from_token(value, ErrorType::UnknownInstruction)),
        }
    }
}

impl Default for Line {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<Result<Token, Error>>> for Line {
    fn from(value: Vec<Result<Token, Error>>) -> Self {
        Self { inner: value }
    }
}

impl From<Token> for Line {
    fn from(value: Token) -> Self {
        Self {
            inner: vec![Ok(value)],
        }
    }
}

impl From<Error> for Line {
    fn from(value: Error) -> Self {
        Self {
            inner: vec![Err(value)],
        }
    }
}

impl Deref for Line {
    type Target = Vec<Result<Token, Error>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Line {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
