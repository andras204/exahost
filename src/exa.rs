mod lexar;

use std::fmt::Display;
use lexar::*;
use serde::{Deserialize, Serialize};

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

    pub fn start(&mut self) {
        println!("==========");
        for l in self.instr_list.clone() { println!("{}", l); }
        println!("==========");
        while (self.instr_ptr as usize) < self.instr_list.len() {
            match self.exec() {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("{}", e);
                    break;
                },
            }
        }
    }

    pub fn exec(&mut self) -> Result<(), &str> {
        let tokens: Vec<Token> = tokenize(self.instr_list[self.instr_ptr as usize].clone()).unwrap();
        self.instr_ptr += 1;
        if tokens[0].instruction().unwrap() == "mark".to_string() { return Ok(()); }
        self.execute_instruction(tokens)
    }

    fn execute_instruction(&mut self, tokens: Vec<Token>) -> Result<(), &str> {
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
            _ => Err("Unknown instruction"),
        }
    }

    fn link(&self, link: &Token) -> Result<(), &str> {
        Ok(())
    }

    fn halt() -> Result<(), &'static str> {
        Err("Halted")
    }

    fn addi(&mut self, op1: &Token, op2: &Token, target: &Token) -> Result<(), &str> {
        let num1 = self.get_number(op1).unwrap();
        let num2 = self.get_number(op2).unwrap();
        let result = Self::clamp_value(num1 + num2);
        *self.get_register_mut(target).unwrap() = result;
        Ok(())
    }

    fn subi(&mut self, op1: &Token, op2: &Token, target: &Token) -> Result<(), &str> {
        let num1 = self.get_number(op1).unwrap();
        let num2 = self.get_number(op2).unwrap();
        let result = Self::clamp_value(num1 - num2);
        *self.get_register_mut(target).unwrap() = result;
        Ok(())
    }

    fn muli(&mut self, op1: &Token, op2: &Token, target: &Token) -> Result<(), &str> {
        let num1 = self.get_number(op1).unwrap();
        let num2 = self.get_number(op2).unwrap();
        let result = Self::clamp_value(num1 * num2);
        *self.get_register_mut(target).unwrap() = result;
        Ok(())
    }

    fn divi(&mut self, op1: &Token, op2: &Token, target: &Token) -> Result<(), &str> {
        let num1 = self.get_number(op1).unwrap();
        let num2 = self.get_number(op2).unwrap();
        let result = Self::clamp_value(num1 / num2);
        *self.get_register_mut(target).unwrap() = result;
        Ok(())
    }

    fn modi(&mut self, op1: &Token, op2: &Token, target: &Token) -> Result<(), &str> {
        let num1 = self.get_number(op1).unwrap();
        let num2 = self.get_number(op2).unwrap();
        let result = Self::clamp_value(num1 % num2);
        *self.get_register_mut(target).unwrap() = result;
        Ok(())
    }

    fn test(&mut self, op1: &Token, comp: &Token, op2: &Token) -> Result<(), &str> {
        // TODO: handle non numbers
        let eval;
        let num1 = self.get_number(op1).unwrap();
        let num2 = self.get_number(op2).unwrap();
        match &comp.comparison().unwrap()[..] {
            "=" => eval = num1 == num2,
            "!=" => eval = num1 != num2,
            ">=" => eval = num1 >= num2,
            "<=" => eval = num1 <= num2,
            ">" => eval = num1 > num2,
            "<" => eval = num1 < num2,
            _ => return Err("Invalid Comparison"),
        }
        if eval { self.reg_t = Register::Number(1); }
        else { self.reg_t = Register::Number(0); }
        Ok(())
    }

    fn jump(&mut self, arg: &Token) -> Result<(), &str> {
        let label = arg.label().unwrap();
        for x in 0..self.instr_list.len() {
            let tokens = split_instruction(self.instr_list[x].to_owned());
            if tokens[0] == "mark" && tokens[1] == label {
                self.instr_ptr = x as u8;
                return Ok(());
            }
        }
        Err("Label not found")
    }

    fn tjmp(&mut self, arg: &Token) -> Result<(), &str> {
        if let Ok(t) = self.reg_t.number() {
            if t != 0 { self.jump(arg) }
            else { Ok(()) }
        }
        else { self.jump(arg) }
    }

    fn fjmp(&mut self, arg: &Token) -> Result<(), &str> {
        if let Ok(t) = self.reg_t.number() {
            if t == 0 { return self.jump(arg); }
        }
        Ok(())
    }

    fn copy(&mut self, value: &Token, target: &Token) -> Result<(), &str> {
        let val: Register;
        match value.token_type {
            TokenType::Register => val = self.get_register_shared(value).unwrap().to_owned(),
            TokenType::Number | TokenType::Keyword => val = Register::from_token(value).unwrap(),
            _ => return Err("Invalid argument type"),
        }
        *self.get_register_mut(target).unwrap() = val;
        Ok(())
    }

    fn print(&self, arg: &Token) -> Result<(), &str>{
        match arg.token_type {
            TokenType::Register => Ok(println!("{}> {}", self.name, *self.get_register_shared(arg).unwrap())),
            TokenType::Keyword => Ok(println!("{}> {}", self.name, arg.keyword().unwrap())),
            _ => Ok(println!("{}> {}", self.name, arg.token))
        }
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
