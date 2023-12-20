use std::{fmt::Display, cmp::Ordering};
use serde::{Deserialize, Serialize};

use crate::{lexar::{*, self}, signal::ExaSignal};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exa {
    pub name: String,
    instr_list: Vec<String>,
    instr_ptr: u8,
    repl_counter: usize,
    x_reg: Register,
    t_reg: Register,
    pub m_reg: Option<Register>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Register {
    Number(i16),
    Keyword(String),
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::Keyword(w) => write!(f, "{}", w),
        }
    }
}

impl PartialOrd for Register {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            Self::Number(n) => match other {
                Self::Number(m) => Some(n.cmp(m)),
                Self::Keyword(_) => None,
            },
            Self::Keyword(k) => match other {
                Self::Number(_) => None,
                Self::Keyword(w) => Some(k.cmp(w)),
            },
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

    fn _keyword(&self) -> Result<String, &str> {
        match self {
            Self::Keyword(w) => Ok(w.to_owned()),
            _ => Err("Not a keyword"),
        }
    }
}

impl Exa {
    pub fn new(name: &str, instruction_list: Vec<String>) -> Result<Self, Vec<String>> {
        let instr_list = lexar::compile(instruction_list)?;
        Ok(Exa {
            name: name.to_string(),
            instr_list,
            instr_ptr: 0,
            repl_counter: 0,
            x_reg: Register::Number(0),
            t_reg: Register::Number(0),
            m_reg: None,
        })
    }

    pub fn send_m(&mut self) -> Option<Register> {
        self.instr_ptr += 1;
        Some(self.m_reg.take().unwrap())
    }

    pub fn exec(&mut self) -> ExaSignal {
        if self.instr_ptr as usize == self.instr_list.len() { return ExaSignal::Err("Out of Instructions".to_string()); }
        let tokens: Vec<Token> = tokenize(self.instr_list[self.instr_ptr as usize].clone()).unwrap();
        if tokens[0].instruction().unwrap() == "mark".to_string() {
            return ExaSignal::Ok;
        }
        match self.execute_instruction(tokens) {
            Ok(s) => {
                self.instr_ptr += 1;
                return s;
            },
            Err(e) => return e,
        }
    }

    fn execute_instruction(&mut self, tokens: Vec<Token>) -> Result<ExaSignal, ExaSignal> {
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
            "repl" => self.repl(&tokens[1]),
            "halt" => Self::halt(),
            "kill" => Self::kill(),
            // misc
            "prnt" => self.print(&tokens[1]),
            "noop" => Self::noop(),
            _ => Err(ExaSignal::Err("Unknown instruction".to_string())),
        }
    }

    fn link(&mut self, link: &Token) -> Result<ExaSignal, ExaSignal> {
        let l = self.get_number(link)?;
        Ok(ExaSignal::Link(l))
    }

    fn halt() -> Result<ExaSignal, ExaSignal> {
        Ok(ExaSignal::Halt)
    }

    fn noop() -> Result<ExaSignal, ExaSignal> {
        Ok(ExaSignal::Ok)
    }

    fn kill() -> Result<ExaSignal, ExaSignal> {
        Ok(ExaSignal::Kill)
    }

    fn repl(&mut self, token: &Token) -> Result<ExaSignal, ExaSignal> {
        let mut clone = self.clone();
        clone.jump(token)?;
        clone.name.push_str(&format!(":{}", self.repl_counter));
        clone.repl_counter = 0;
        self.repl_counter += 1;
        Ok(ExaSignal::Repl(clone))
    }

    fn addi(&mut self, op1: &Token, op2: &Token, target: &Token) -> Result<ExaSignal, ExaSignal> {
        let num1 = self.get_number(op1)?;
        let num2 = self.get_number(op2)?;
        self.put_value(Register::Number(num1 + num2), target)?;
        Ok(ExaSignal::Ok)
    }

    fn subi(&mut self, op1: &Token, op2: &Token, target: &Token) -> Result<ExaSignal, ExaSignal> {
        let num1 = self.get_number(op1)?;
        let num2 = self.get_number(op2)?;
        self.put_value(Register::Number(num1 - num2), target)?;
        Ok(ExaSignal::Ok)
    }

    fn muli(&mut self, op1: &Token, op2: &Token, target: &Token) -> Result<ExaSignal, ExaSignal> {
        let num1 = self.get_number(op1)?;
        let num2 = self.get_number(op2)?;
        self.put_value(Register::Number(num1 * num2), target)?;
        Ok(ExaSignal::Ok)
    }

    fn divi(&mut self, op1: &Token, op2: &Token, target: &Token) -> Result<ExaSignal, ExaSignal> {
        let num1 = self.get_number(op1)?;
        let num2 = self.get_number(op2)?;
        self.put_value(Register::Number(num1 / num2), target)?;
        Ok(ExaSignal::Ok)
    }

    fn modi(&mut self, op1: &Token, op2: &Token, target: &Token) -> Result<ExaSignal, ExaSignal> {
        let num1 = self.get_number(op1)?;
        let num2 = self.get_number(op2)?;
        self.put_value(Register::Number(num1 % num2), target)?;
        Ok(ExaSignal::Ok)
    }

    fn test(&mut self, op1: &Token, comp: &Token, op2: &Token) -> Result<ExaSignal, ExaSignal> {
        let eval;
        let v1 = self.get_value(op1)?;
        let v2 = self.get_value(op2)?;
        match &comp.comparison().unwrap()[..] {
            "=" => eval = v1 == v2,
            "!=" => eval = v1 != v2,
            ">=" => eval = v1 >= v2,
            "<=" => eval = v1 <= v2,
            ">" => eval = v1 > v2,
            "<" => eval = v1 < v2,
            _ => return Err(ExaSignal::Err("Invalid Comparison".to_string())),
        }
        if eval { self.t_reg = Register::Number(1); }
        else { self.t_reg = Register::Number(0); }
        Ok(ExaSignal::Ok)
    }

    fn jump(&mut self, arg: &Token) -> Result<ExaSignal, ExaSignal> {
        let label = arg.label().unwrap();
        for x in 0..self.instr_list.len() {
            let tokens = split_instruction(self.instr_list[x].to_owned());
            if tokens[0] == "mark" && tokens[1] == label {
                self.instr_ptr = x as u8;
                return Ok(ExaSignal::Ok);
            }
        }
        Err(ExaSignal::Err("Label not found".to_string()))
    }

    fn tjmp(&mut self, arg: &Token) -> Result<ExaSignal, ExaSignal> {
        if let Ok(t) = self.t_reg.number() {
            if t != 0 { self.jump(arg) }
            else { Ok(ExaSignal::Ok) }
        }
        else { self.jump(arg) }
    }

    fn fjmp(&mut self, arg: &Token) -> Result<ExaSignal, ExaSignal> {
        if let Ok(t) = self.t_reg.number() {
            if t == 0 { return self.jump(arg) }
        }
        Ok(ExaSignal::Ok)
    }

    fn copy(&mut self, value: &Token, target: &Token) -> Result<ExaSignal, ExaSignal> {
        let val: Register;
        match value.token_type {
            TokenType::Register => val = self.get_value(value)?,
            TokenType::Number | TokenType::Keyword => val = Register::from_token(value).unwrap(),
            _ => return Err(ExaSignal::Err("Invalid argument type".to_string())),
        }
        self.put_value(val, target)?;
        Ok(ExaSignal::Ok)
    }

    fn print(&mut self, arg: &Token) -> Result<ExaSignal, ExaSignal> {
        let name = self.name.clone();
        match arg.token_type {
            TokenType::Register => println!("{}> {}", name, self.get_value(arg)?),
            TokenType::Keyword => println!("{}> {}", name, arg.keyword().unwrap()),
            TokenType::Number => println!("{}> {}", name, arg.number().unwrap()),
            _ => println!("{}> {}", name, arg.token),
        }
        Ok(ExaSignal::Ok)
    }

    fn put_value(&mut self, value: Register, target: &Token) -> Result<(), ExaSignal> {
        let value = match value {
            Register::Number(n) => Register::Number(n.clamp(-9999, 9999)),
            Register::Keyword(w) => Register::Keyword(w),
        };
        match target.register().unwrap() {
            'x' => *self.get_register_mut('x').unwrap() = value,
            't' => *self.get_register_mut('t').unwrap() = value,
            'm' => {
                match self.m_reg {
                    Some(_) => return Err(ExaSignal::Tx),
                    None => {
                        self.m_reg = Some(value);
                        return Err(ExaSignal::Tx);
                    },
                }
            },
            'f' => panic!("File register not implemented"),
            _ => return Err(ExaSignal::Err("Invalid register".to_string())),
        }
        Ok(())
    }

    fn get_value(&mut self, target: &Token) -> Result<Register, ExaSignal> {
        match target.token_type {
            TokenType::Register => (),
            _ => return Ok(Register::from_token(target).unwrap()),
        }
        match target.register().unwrap() {
            'x' => Ok(self.get_register_ref('x').unwrap().to_owned()),
            't' => Ok(self.get_register_ref('t').unwrap().to_owned()),
            'm' => {
                match self.m_reg.take() {
                    Some(r) => Ok(r),
                    None => Err(ExaSignal::Rx)
                }
            },
            'f' => panic!("File register not implemented"), //TODO: implement
            _ => Err(ExaSignal::Err("Invalid register".to_string())),
        }
    }

    fn get_register_ref(&self, reg: char) -> Result<&Register, &str> {
        match reg {
            'x' => Ok(&self.x_reg),
            't' => Ok(&self.t_reg),
            _ => Err("Invalid register"),
        }
    }

    fn get_register_mut(&mut self, reg: char) -> Result<&mut Register, &str> {
        match reg {
            'x' => Ok(&mut self.x_reg),
            't' => Ok(&mut self.t_reg),
            _ => Err("Invalid register"),
        }
    }

    fn get_number<'a>(&'a mut self, arg: &'a Token) -> Result<i16, ExaSignal> {
        match arg.token_type {
            TokenType::Register =>  {
                    let result = self.get_value(arg)?;
                    match result {
                        Register::Number(n) => Ok(n),
                        Register::Keyword(_) => Err(ExaSignal::Err("Not a number".to_string())),
                    }
                },
            TokenType::Number => return Ok(arg.number().unwrap()),
            _ => return Err(ExaSignal::Err("Not a number".to_string())),
        }
    }
}
