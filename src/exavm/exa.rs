use std::fmt::Display;
use serde::{Deserialize, Serialize};

use crate::lexar::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exa {
    name: String,
    instr_list: Vec<String>,
    instr_ptr: u8,
    reg_x: Register,
    reg_t: Register,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Register {
    Number(i16),
    Keyword(String),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExaStatus {
    Ok,
    Halt,
    Err(String),
    LinkRq(i16),
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::Keyword(w) => write!(f, "{}", w),
        }
    }
}

impl Register {
    fn from_token(token: &Token) -> Result<Register, &'static str> {
        match token.token_type {
            TokenType::Number => Ok(Register::Number(token.number().unwrap())),
            TokenType::Keyword => Ok(Register::Keyword(token.keyword().unwrap())),
            _ => Err("Invalid token type")
        }
    }

    fn number(&self) -> Result<i16, &str> {
        match self {
            Self::Number(n) => Ok(n.to_owned()),
            _ => Err("Not a number"),
        }
    }

    fn keyword(&self) -> Result<String, &str> {
        match self {
            Self::Keyword(w) => Ok(w.to_owned()),
            _ => Err("Not a keyword"),
        }
    }
}

impl Exa {
    pub fn new(name: String, code: Vec<String>) -> Result<Exa, Vec<String>> {
        match compile(code) {
            Ok(instr_list) => Ok(Exa {
                name,
                instr_list,
                instr_ptr: 0,
                reg_x: Register::Number(0),
                reg_t: Register::Number(0),
            }),
            Err(err_list) => Err(err_list),
        }
    }

    pub fn exec(&mut self) -> ExaStatus {
        if self.instr_ptr as usize == self.instr_list.len() { return ExaStatus::Err("Out of Instructions".to_string()); }
        let tokens: Vec<Token> = tokenize(self.instr_list[self.instr_ptr as usize].clone()).unwrap();
        self.instr_ptr += 1;
        if tokens[0].instruction().unwrap() == "mark".to_string() { return ExaStatus::Ok; }
        self.execute_instruction(tokens)
    }

    fn execute_instruction(&mut self, tokens: Vec<Token>) -> ExaStatus {
        match &tokens[0].instruction().unwrap()[..] {
            // I/O
            "copy" => self.copy(&tokens[1], &tokens[2]),
            // math
            "addi" => self.addi(&tokens[1], &tokens[2], &tokens[3]),
            "subi" => self.subi(&tokens[1], &tokens[2], &tokens[3]),
            "muli" => self.muli(&tokens[1], &tokens[2], &tokens[3]),
            "divi" => self.divi(&tokens[1], &tokens[2], &tokens[3]),
            "modi" => self.modi(&tokens[1], &tokens[2], &tokens[3]),
            // test
            "test" => self.test(&tokens[1], &tokens[2], &tokens[3]),
            // jumps
            "jump" => self.jump(&tokens[1]),
            "tjmp" => self.tjmp(&tokens[1]),
            "fjmp" => self.fjmp(&tokens[1]),
            // lifecycle
            "link" => self.link(&tokens[1]),
            "halt" => Self::halt(),
            // misc
            "prnt" => self.print(&tokens[1]),
            _ => ExaStatus::Err("Unknown instruction".to_string()),
        }
    }

    fn link(&self, link: &Token) -> ExaStatus {
        match self.get_number(link) {
            Ok(l) => ExaStatus::LinkRq(l),
            Err(e) => ExaStatus::Err(e.to_string()),
        }
    }

    fn halt() -> ExaStatus {
        ExaStatus::Halt
    }

    fn addi(&mut self, op1: &Token, op2: &Token, target: &Token) -> ExaStatus {
        let num1 = match self.get_number(op1) {
            Ok(n) => n,
            Err(e) => return ExaStatus::Err(e.to_string()),
        };
        let num2 = match self.get_number(op2) {
            Ok(n) => n,
            Err(e) => return ExaStatus::Err(e.to_string()),
        };
        *self.get_register_mut(target).unwrap() = Register::Number(num1 + num2);
        ExaStatus::Ok
    }

    fn subi(&mut self, op1: &Token, op2: &Token, target: &Token) -> ExaStatus {
        let num1 = match self.get_number(op1) {
            Ok(n) => n,
            Err(e) => return ExaStatus::Err(e.to_string()),
        };
        let num2 = match self.get_number(op2) {
            Ok(n) => n,
            Err(e) => return ExaStatus::Err(e.to_string()),
        };
        *self.get_register_mut(target).unwrap() = Register::Number(num1 - num2);
        ExaStatus::Ok
    }

    fn muli(&mut self, op1: &Token, op2: &Token, target: &Token) -> ExaStatus {
        let num1 = match self.get_number(op1) {
            Ok(n) => n,
            Err(e) => return ExaStatus::Err(e.to_string()),
        };
        let num2 = match self.get_number(op2) {
            Ok(n) => n,
            Err(e) => return ExaStatus::Err(e.to_string()),
        };
        *self.get_register_mut(target).unwrap() = Register::Number(num1 * num2);
        ExaStatus::Ok
    }

    fn divi(&mut self, op1: &Token, op2: &Token, target: &Token) -> ExaStatus {
        let num1 = match self.get_number(op1) {
            Ok(n) => n,
            Err(e) => return ExaStatus::Err(e.to_string()),
        };
        let num2 = match self.get_number(op2) {
            Ok(n) => n,
            Err(e) => return ExaStatus::Err(e.to_string()),
        };
        *self.get_register_mut(target).unwrap() = Register::Number(num1 / num2);
        ExaStatus::Ok
    }

    fn modi(&mut self, op1: &Token, op2: &Token, target: &Token) -> ExaStatus {
        let num1 = match self.get_number(op1) {
            Ok(n) => n,
            Err(e) => return ExaStatus::Err(e.to_string()),
        };
        let num2 = match self.get_number(op2) {
            Ok(n) => n,
            Err(e) => return ExaStatus::Err(e.to_string()),
        };
        *self.get_register_mut(target).unwrap() = Register::Number(num1 % num2);
        ExaStatus::Ok
    }

    fn test(&mut self, op1: &Token, comp: &Token, op2: &Token) -> ExaStatus {
        // TODO: handle non numbers
        let eval;
        let num1 = match self.get_number(op1) {
            Ok(n) => n,
            Err(e) => return ExaStatus::Err(e.to_string()),
        };
        let num2 = match self.get_number(op2) {
            Ok(n) => n,
            Err(e) => return ExaStatus::Err(e.to_string()),
        };
        match &comp.comparison().unwrap()[..] {
            "=" => eval = num1 == num2,
            "!=" => eval = num1 != num2,
            ">=" => eval = num1 >= num2,
            "<=" => eval = num1 <= num2,
            ">" => eval = num1 > num2,
            "<" => eval = num1 < num2,
            _ => return ExaStatus::Err("Invalid Comparison".to_string()),
        }
        if eval { self.reg_t = Register::Number(1); }
        else { self.reg_t = Register::Number(0); }
        ExaStatus::Ok
    }

    fn jump(&mut self, arg: &Token) -> ExaStatus {
        let label = arg.label().unwrap();
        for x in 0..self.instr_list.len() {
            let tokens = split_instruction(self.instr_list[x].to_owned());
            if tokens[0] == "mark" && tokens[1] == label {
                self.instr_ptr = x as u8;
                return ExaStatus::Ok;
            }
        }
        ExaStatus::Err("Label not found".to_string())
    }

    fn tjmp(&mut self, arg: &Token) -> ExaStatus {
        if let Ok(t) = self.reg_t.number() {
            if t != 0 { self.jump(arg) }
            else { ExaStatus::Ok }
        }
        else { self.jump(arg) }
    }

    fn fjmp(&mut self, arg: &Token) -> ExaStatus {
        if let Ok(t) = self.reg_t.number() {
            if t == 0 { return self.jump(arg); }
        }
        ExaStatus::Ok
    }

    fn copy(&mut self, value: &Token, target: &Token) -> ExaStatus {
        let val: Register;
        match value.token_type {
            TokenType::Register => val = self.get_register_shared(value).unwrap().to_owned(),
            TokenType::Number | TokenType::Keyword => val = Register::from_token(value).unwrap(),
            _ => return ExaStatus::Err("Invalid argument type".to_string()),
        }
        *self.get_register_mut(target).unwrap() = val;
        ExaStatus::Ok
    }

    fn print(&self, arg: &Token) -> ExaStatus {
        match arg.token_type {
            TokenType::Register => println!("{}> {}", self.name, *self.get_register_shared(arg).unwrap()),
            TokenType::Keyword => println!("{}> {}", self.name, arg.keyword().unwrap()),
            _ => println!("{}> {}", self.name, arg.token),
        }
        ExaStatus::Ok
    }

    fn clamp_value(mut val: i16) -> Register {
        if val > 9999 { val = 9999; }
        if val < -9999 { val = -9999; }
        Register::Number(val)
    }

    fn get_register_shared(&self, arg: &Token) -> Result<&Register, &str> {
        match arg.register().unwrap() {
            'x' => Ok(&self.reg_x),
            't' => Ok(&self.reg_t),
            _ => Err("Invalid register"),
        }
    }

    fn get_register_mut(&mut self, arg: &Token) -> Result<&mut Register, &str> {
        match arg.register().unwrap() {
            'x' => Ok(&mut self.reg_x),
            't' => Ok(&mut self.reg_t),
            _ => Err("Invalid register"),
        }
    }

    fn get_keyword<'a>(&'a self, arg: &'a Token) -> Result<String, &str> {
        match arg.token_type {
            TokenType::Register => {
                match arg.register().unwrap() {
                    'x' => self.reg_x.keyword(),
                    't' => self.reg_t.keyword(),
                    _ => return Err("Invalid register"),
                }
            }
            TokenType::Keyword => return arg.keyword(),
            _ => return Err("Not a keyword"),
        }
    }

    fn get_number<'a>(&'a self, arg: &'a Token) -> Result<i16, &str> {
        match arg.token_type {
            TokenType::Register => {
                match arg.register().unwrap() {
                    'x' => self.reg_x.number(),
                    't' => self.reg_t.number(),
                    _ => return Err("Invalid register"),
                }
            }
            TokenType::Number => return arg.number(),
            _ => return Err("Not a number"),
        }
    }
}
