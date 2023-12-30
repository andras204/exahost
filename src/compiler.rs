use std::collections::HashMap;

use crate::exa::{Arg, Comp, RegLabel, Instruction};

#[derive(Debug, Clone)]
pub struct Token {
    pub token: String,
    pub token_type: TokenType,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum TokenType {
    Instruction,
    Number,
    Keyword,
    Register,
    Label,
    Comparison,
    Comment,
}

impl Token {
    pub fn new(token: String, token_type: TokenType) -> Token {
        Token {
            token,
            token_type,
        }
    }

    pub fn to_instruction(&self) -> Result<Instruction, String> {
        match self.token_type {
            TokenType::Instruction => match &self.token[..] {
                "copy" => Ok(Instruction::Copy),

                "addi" => Ok(Instruction::Addi),
                "subi" => Ok(Instruction::Subi),
                "muli" => Ok(Instruction::Muli),
                "divi" => Ok(Instruction::Divi),
                "modi" => Ok(Instruction::Modi),
                "swiz" => Ok(Instruction::Swiz),

                "test" => Ok(Instruction::Test),

                "mark" => Ok(Instruction::Mark),
                "jump" => Ok(Instruction::Jump),
                "tjmp" => Ok(Instruction::Tjmp),
                "fjmp" => Ok(Instruction::Fjmp),

                "link" => Ok(Instruction::Link),
                "repl" => Ok(Instruction::Repl),
                "halt" => Ok(Instruction::Halt),
                "kill" => Ok(Instruction::Kill),

                "noop" => Ok(Instruction::Noop),
                "prnt" => Ok(Instruction::Prnt),
                _ => Err("unknown instruction".to_string()),
            },
            _ => Err("not an instruction".to_string()),
        }
    }

    pub fn to_arg(&self) -> Result<Arg, &str> {
        match self.token_type {
            TokenType::Instruction => Err("instruction cannot be parsed as arg"),
            TokenType::Register => match &self.token[..] {
                "x" => Ok(Arg::Register(RegLabel::X)),
                "t" => Ok(Arg::Register(RegLabel::T)),
                "f" => Ok(Arg::Register(RegLabel::F)),
                "m" => Ok(Arg::Register(RegLabel::M)),
                s => {
                    if s.starts_with('#') { Ok(Arg::Register(RegLabel::H(s.to_string()))) }
                    else { Err("invalid register label") }
                },
            }
            TokenType::Number => Ok(Arg::Number(self.token.parse::<i16>().unwrap())),
            TokenType::Comparison => match &self.token[..] {
                "=" => Ok(Arg::Comp(Comp::Eq)),
                ">" => Ok(Arg::Comp(Comp::Gt)),
                "<" => Ok(Arg::Comp(Comp::Lt)),
                ">=" => Ok(Arg::Comp(Comp::Ge)),
                "<=" => Ok(Arg::Comp(Comp::Le)),
                "!=" => Ok(Arg::Comp(Comp::Ne)),
                _ => Err("invalid comparison operator"),
            }
            TokenType::Keyword => Ok(Arg::Keyword(self.token.clone())),
            TokenType::Label => Ok(Arg::Label(self.token.clone())),
            TokenType::Comment => Err("comment cannot be parsed as arg"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Compiler {
    sigs: HashMap<String, Vec<Vec<TokenType>>>,
    comps: Vec<String>,
    keyword_delim: char,
    multi_m: bool,
}

impl Compiler {
    pub fn new() -> Self {
        let config = CompilerConfig::default();
        Self {
            sigs: config.sigs,
            comps: config.comps,
            keyword_delim: config.keyword_delim,
            multi_m: config.multi_m,
        }
    }

    pub fn with_config(config: CompilerConfig) -> Self {
        Self {
            sigs: config.sigs,
            comps: config.comps,
            keyword_delim: config.keyword_delim,
            multi_m: config.multi_m,
        }
    }

    pub fn compile(&self, source: Vec<String>) -> Result<Vec<(Instruction, Option<Vec<Arg>>)>, Vec<String>> {
        let mut errs: Vec<String> = Vec::new();
        let mut compiled: Vec<(Instruction, Option<Vec<Arg>>)> = Vec::with_capacity(source.len());
        if source.len() == 0 { errs.push("nothing to compile".to_string()); }
        for x in 0..source.len() {
            let split = self.split_line(source[x].clone());
            if split[0].eq("note") { continue; }
            let tokens = match self.tokenize(split) {
                Ok(ts) => ts,
                Err(e) => {
                    errs.push(format!("{} on line {}", e, x));
                    continue;
                },
            };
            let instr = match self.tokens_to_instructions(tokens) {
                Ok(i) => i,
                Err(e) => {
                    errs.push(format!("{} on line {}", e, x));
                    continue;
                },
            };
            compiled.push(instr);
        }
        if errs.len() > 0 { return Err(errs); }
        Ok(compiled)
    }

    fn split_line(&self, line: String) -> Vec<String> {
        if line.starts_with("note") || line.starts_with(";") {
            return vec![ "note".to_string() ];
        }
    
        let mut sliced: Vec<String> = vec![ line[..4].to_string() ];
    
        if line.len() > 5 {
            let arg_slice = &line[5..];
            let mut t_begin: usize = 0;
            let mut mid_word: bool = false;
            let mut x = 0;
            while x < arg_slice.len() {
                let curr_char = arg_slice.chars().nth(x).unwrap();
                if curr_char == self.keyword_delim {
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
        sliced
    }

    // also sig match
    fn tokenize(&self, strs: Vec<String>) -> Result<Vec<Token>, String> {
        let instr = Token::new(strs[0].clone(), TokenType::Instruction);
        if strs.len() <= 1 { return Ok(vec![instr]); }
        let sig = self.get_sig(&strs[0])?;
        let mut tokens = vec![instr];
        for x in 0..(strs.len() - 1) {
            for y in 0..sig[x].len() {
                match self.try_parse(strs[x + 1].clone(), &sig[x][y]) {
                    Ok(t) => {
                        tokens.push(t);
                        break;
                    },
                    Err(_) => if y == sig[x].len() - 1 {
                        return Err("failed to parse argument".to_string());
                    },
                }
            }
        }
        Ok(tokens)
    }

    fn tokens_to_instructions(&self, tokens: Vec<Token>) -> Result<(Instruction, Option<Vec<Arg>>), String> {
        let instr = tokens[0].to_instruction()?;
        if tokens.len() <= 1 { return Ok((instr, None)); }
        let mut args: Vec<Arg> = Vec::with_capacity(tokens.len() - 1);
        for token in &tokens[1..] {
            args.push(token.to_arg()?);
        }
        let mut ms = 0;
        for a in args.iter() {
            if a.is_reg_m() { ms += 1; }
        }
        if !self.multi_m && ms > 1 { return Err("multiple M use".to_string()); }
        Ok((instr, Some(args)))
    }

    fn get_sig(&self, instr: &String) -> Result<Vec<Vec<TokenType>>, String> {
        match self.sigs.get(instr) {
            Some(s) => Ok(s.clone()),
            None => Err("unknown instruction".to_string()),
        }
    }

    fn try_parse(&self, str: String, tt: &TokenType) -> Result<Token, String> {
        match tt {
            TokenType::Register => {
                if ("xtfm".contains(&str) && str.len() == 1) || str.starts_with('#') {
                    return Ok(Token::new(str, tt.clone()));
                }
                else { return Err("not a register".to_string()); }
            },
            TokenType::Number => match str.parse::<i16>() {
                Ok(_) => return Ok(Token::new(str, tt.clone())),
                Err(_) => return Err("not a number".to_string()),
            },
            TokenType::Comparison => {
                if self.comps.contains(&str) {
                    return Ok(Token::new(str, tt.clone()));
                }
                else { return Err("not a comparison".to_string()); }
            }
            TokenType::Keyword => {
                if str.starts_with(self.keyword_delim) && str.ends_with(self.keyword_delim) {
                    return Ok(Token::new(str, tt.clone()));
                }
                else { return Err("not a keyword".to_string()); }
            },
            TokenType::Label => return Ok(Token::new(str, tt.clone())),
            _ => Err("cannot parse type".to_string()),
        }
    }
}

pub struct CompilerConfig {
    sigs: HashMap<String, Vec<Vec<TokenType>>>,
    comps: Vec<String>,
    keyword_delim: char,
    multi_m: bool,
}

impl CompilerConfig {
    pub fn default() -> Self {
        let r = vec![TokenType::Register];
        let rn = vec![TokenType::Register, TokenType::Number];
        let c = vec![TokenType::Comparison];
        let l = vec![TokenType::Label];

        let mut sigs = HashMap::new();

        sigs.insert("copy".to_string(), vec![rn.clone(), r.clone()]);

        sigs.insert("addi".to_string(), vec![rn.clone(), rn.clone(), r.clone()]);
        sigs.insert("subi".to_string(), vec![rn.clone(), rn.clone(), r.clone()]);
        sigs.insert("muli".to_string(), vec![rn.clone(), rn.clone(), r.clone()]);
        sigs.insert("divi".to_string(), vec![rn.clone(), rn.clone(), r.clone()]);
        sigs.insert("modi".to_string(), vec![rn.clone(), rn.clone(), r.clone()]);
        sigs.insert("swiz".to_string(), vec![rn.clone(), rn.clone(), r.clone()]);

        sigs.insert("test".to_string(), vec![rn.clone(), c.clone(), rn.clone()]);

        sigs.insert("mark".to_string(), vec![l.clone()]);

        sigs.insert("jump".to_string(), vec![l.clone()]);
        sigs.insert("tjmp".to_string(), vec![l.clone()]);
        sigs.insert("fjmp".to_string(), vec![l.clone()]);

        sigs.insert("repl".to_string(), vec![l.clone()]);

        sigs.insert("link".to_string(), vec![rn.clone()]);

        sigs.insert("kill".to_string(), vec![]);
        sigs.insert("halt".to_string(), vec![]);
        sigs.insert("noop".to_string(), vec![]);

        let comps = vec!["=", ">", "<"].into_iter()
            .map(|s| s.to_string()).collect();

        Self {
            sigs,
            comps,
            keyword_delim: '\'',
            multi_m: false,
        }
    }

    pub fn extended() -> Self {
        let r = vec![TokenType::Register];
        let rn = vec![TokenType::Register, TokenType::Number];
        let vari = vec![TokenType::Register, TokenType::Number, TokenType::Keyword];
        let c = vec![TokenType::Comparison];
        let l = vec![TokenType::Label];

        let mut sigs = HashMap::new();

        sigs.insert("copy".to_string(), vec![vari.clone(), r.clone()]);

        sigs.insert("addi".to_string(), vec![rn.clone(), rn.clone(), r.clone()]);
        sigs.insert("subi".to_string(), vec![rn.clone(), rn.clone(), r.clone()]);
        sigs.insert("muli".to_string(), vec![rn.clone(), rn.clone(), r.clone()]);
        sigs.insert("divi".to_string(), vec![rn.clone(), rn.clone(), r.clone()]);
        sigs.insert("modi".to_string(), vec![rn.clone(), rn.clone(), r.clone()]);
        sigs.insert("swiz".to_string(), vec![rn.clone(), rn.clone(), r.clone()]);

        sigs.insert("test".to_string(), vec![vari.clone(), c.clone(), vari.clone()]);

        sigs.insert("mark".to_string(), vec![l.clone()]);

        sigs.insert("jump".to_string(), vec![l.clone()]);
        sigs.insert("tjmp".to_string(), vec![l.clone()]);
        sigs.insert("fjmp".to_string(), vec![l.clone()]);

        sigs.insert("repl".to_string(), vec![l.clone()]);

        sigs.insert("link".to_string(), vec![rn.clone()]);

        sigs.insert("kill".to_string(), vec![]);
        sigs.insert("halt".to_string(), vec![]);
        sigs.insert("noop".to_string(), vec![]);

        sigs.insert("prnt".to_string(), vec![vari.clone()]);

        let comps = vec!["=", ">", "<", ">=", "<=", "!="].into_iter()
            .map(|s| s.to_string()).collect();

        Self {
            sigs,
            comps,
            keyword_delim: '\'',
            multi_m: true,
        }
    }

    pub fn custom(
        extra_instructions: bool,
        keyword_literals: bool,
        full_comps: bool,
        keyword_delim: char,
        multi_m: bool,
    ) -> Self {
        let r = vec![TokenType::Register];
        let rn = vec![TokenType::Register, TokenType::Number];
        let vari = match keyword_literals {
            true => vec![TokenType::Register, TokenType::Number, TokenType::Keyword],
            false => rn.clone(),
        };
        let c = vec![TokenType::Comparison];
        let l = vec![TokenType::Label];

        let mut sigs = HashMap::new();

        sigs.insert("copy".to_string(), vec![vari.clone(), r.clone()]);

        sigs.insert("addi".to_string(), vec![rn.clone(), rn.clone(), r.clone()]);
        sigs.insert("subi".to_string(), vec![rn.clone(), rn.clone(), r.clone()]);
        sigs.insert("muli".to_string(), vec![rn.clone(), rn.clone(), r.clone()]);
        sigs.insert("divi".to_string(), vec![rn.clone(), rn.clone(), r.clone()]);
        sigs.insert("modi".to_string(), vec![rn.clone(), rn.clone(), r.clone()]);
        sigs.insert("swiz".to_string(), vec![rn.clone(), rn.clone(), r.clone()]);

        sigs.insert("test".to_string(), vec![vari.clone(), c.clone(), vari.clone()]);

        sigs.insert("mark".to_string(), vec![l.clone()]);

        sigs.insert("jump".to_string(), vec![l.clone()]);
        sigs.insert("tjmp".to_string(), vec![l.clone()]);
        sigs.insert("fjmp".to_string(), vec![l.clone()]);

        sigs.insert("repl".to_string(), vec![l.clone()]);

        sigs.insert("link".to_string(), vec![rn.clone()]);

        sigs.insert("kill".to_string(), vec![]);
        sigs.insert("halt".to_string(), vec![]);
        sigs.insert("noop".to_string(), vec![]);

        if extra_instructions {
            sigs.insert("prnt".to_string(), vec![vari.clone()]);
        }

        let comps = match full_comps {
            true => vec!["=", ">", "<", ">=", "<=", "!="].into_iter().map(|s| s.to_string()).collect(),
            false => vec!["=", ">", "<"].into_iter().map(|s| s.to_string()).collect(),
        };

        Self {
            sigs,
            comps,
            keyword_delim,
            multi_m,
        }
    }
}
