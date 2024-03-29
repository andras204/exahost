use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    fmt::{self, Display},
};
use strum::{Display, EnumString};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExaResult {
    Error(Error),
    VMRequest(VMRequest),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Error {
    OutOfInstructions,
    DivideByZero,
    MathWithKeywords,
    LabelNotFound,
    InvalidFRegisterAccess,
    InvalidHWRegisterAccess,
    InvalidFileAccess,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let e = match self {
            Self::OutOfInstructions => "Out of Instructions",
            Self::DivideByZero => "Divide by Zero",
            Self::MathWithKeywords => "Math with Keyword",
            Self::LabelNotFound => "Label Not Found",
            Self::InvalidFRegisterAccess => "Invalid F Register Access",
            Self::InvalidHWRegisterAccess => "Invalid Hardware Register Access",
            Self::InvalidFileAccess => "Invalid File Access",
        };
        write!(f, "{}", e)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VMRequest {
    Tx,
    Rx,
    Halt,
    Kill,
    Link(i16),
    Repl(Box<Exa>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Exa {
    pub name: String,
    pub instr_list: Vec<(Instruction, Option<Vec<Arg>>)>,
    pub instr_ptr: u8,
    pub repl_counter: usize,
    pub reg_x: Register,
    pub reg_t: Register,
    pub reg_m: Option<Register>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[strum(serialize_all = "lowercase")]
pub enum Instruction {
    Copy,

    Addi,
    Subi,
    Muli,
    Divi,
    Modi,
    Swiz,

    Test,

    Mark,
    Jump,
    Fjmp,
    Tjmp,

    Link,
    Repl,
    Halt,
    Kill,

    Rand,

    Noop,
    Prnt,
}

impl Exa {
    fn link(&mut self, args: Vec<Arg>) -> Result<(), ExaResult> {
        let l = self.get_number(&args[0])?;
        Err(ExaResult::VMRequest(VMRequest::Link(l)))
    }

    fn halt() -> Result<(), ExaResult> {
        Err(ExaResult::VMRequest(VMRequest::Halt))
    }

    fn noop() -> Result<(), ExaResult> {
        Ok(())
    }

    fn kill() -> Result<(), ExaResult> {
        Err(ExaResult::VMRequest(VMRequest::Kill))
    }

    fn repl(&mut self, args: Vec<Arg>) -> Result<(), ExaResult> {
        let mut clone = self.clone();
        clone.jump(args)?;
        clone.name.push_str(&format!(":{}", self.repl_counter));
        clone.repl_counter = 0;
        self.repl_counter += 1;
        Err(ExaResult::VMRequest(VMRequest::Repl(Box::new(clone))))
    }

    fn rand(&mut self, args: Vec<Arg>) -> Result<(), ExaResult> {
        let num1 = self.get_number(&args[0])?;
        let num2 = self.get_number(&args[1])?;
        let mut rng = rand::thread_rng();
        if num1 < num2 {
            self.put_value(Register::from(rng.gen_range(num1..=num2)), &args[2])?;
        } else {
            self.put_value(Register::from(rng.gen_range(num2..=num1)), &args[2])?;
        }
        Ok(())
    }

    fn addi(&mut self, args: Vec<Arg>) -> Result<(), ExaResult> {
        let num1 = self.get_number(&args[0])?;
        let num2 = self.get_number(&args[1])?;
        self.put_value(Register::Number(num1 + num2), &args[2])?;
        Ok(())
    }

    fn subi(&mut self, args: Vec<Arg>) -> Result<(), ExaResult> {
        let num1 = self.get_number(&args[0])?;
        let num2 = self.get_number(&args[1])?;
        self.put_value(Register::Number(num1 - num2), &args[2])?;
        Ok(())
    }

    fn muli(&mut self, args: Vec<Arg>) -> Result<(), ExaResult> {
        let num1 = self.get_number(&args[0])?;
        let num2 = self.get_number(&args[1])?;
        self.put_value(Register::Number(num1 * num2), &args[2])?;
        Ok(())
    }

    fn divi(&mut self, args: Vec<Arg>) -> Result<(), ExaResult> {
        let num1 = self.get_number(&args[0])?;
        let num2 = self.get_number(&args[1])?;
        self.put_value(Register::Number(num1 / num2), &args[2])?;
        Ok(())
    }

    fn modi(&mut self, args: Vec<Arg>) -> Result<(), ExaResult> {
        let num1 = self.get_number(&args[0])?;
        let num2 = self.get_number(&args[1])?;
        self.put_value(Register::Number(num1 % num2), &args[2])?;
        Ok(())
    }

    fn swiz(&mut self, args: Vec<Arg>) -> Result<(), ExaResult> {
        let num1 = self.get_number(&args[0])?;
        let num2 = self.get_number(&args[1])?;
        let mut result = 0;
        for x in 1..5 {
            let mask = match (num2.abs() % 10i16.pow(x) / 10i16.pow(x - 1)) as u32 {
                1 => 1,
                2 => 2,
                3 => 3,
                4 => 4,
                _ => continue,
            };
            result += (num1.abs() % 10i16.pow(mask) / 10i16.pow(mask - 1)) * 10i16.pow(x - 1);
        }
        result *= num1.signum() * num2.signum();
        self.put_value(Register::Number(result), &args[2])?;
        Ok(())
    }

    fn test(&mut self, args: Vec<Arg>) -> Result<(), ExaResult> {
        let v1 = self.get_value(&args[0])?;
        let v2 = self.get_value(&args[2])?;
        let eval = match args[1].comp().unwrap() {
            Comp::Eq => v1 == v2,
            Comp::Gt => v1 > v2,
            Comp::Lt => v1 < v2,
            Comp::Ge => v1 >= v2,
            Comp::Le => v1 <= v2,
            Comp::Ne => v1 != v2,
        };
        self.reg_t = Register::Number(eval as i16);
        Ok(())
    }

    fn jump(&mut self, args: Vec<Arg>) -> Result<(), ExaResult> {
        for x in 0..self.instr_list.len() {
            if self.instr_list[x].0 != Instruction::Mark {
                continue;
            }
            if self.instr_list[x].1.clone().unwrap()[0] == args[0] {
                self.instr_ptr = x as u8;
                return Ok(());
            }
        }
        Err(ExaResult::Error(Error::LabelNotFound))
    }

    fn tjmp(&mut self, args: Vec<Arg>) -> Result<(), ExaResult> {
        if self.get_value(&Arg::reg_t()).unwrap() != Register::Number(0) {
            self.jump(args)
        } else {
            Ok(())
        }
    }

    fn fjmp(&mut self, args: Vec<Arg>) -> Result<(), ExaResult> {
        if self.get_value(&Arg::reg_t()).unwrap() == Register::Number(0) {
            self.jump(args)
        } else {
            Ok(())
        }
    }

    fn copy(&mut self, args: Vec<Arg>) -> Result<(), ExaResult> {
        let val = self.get_value(&args[0])?;
        self.put_value(val, &args[1])?;
        Ok(())
    }

    fn print(&mut self, args: Vec<Arg>) -> Result<(), ExaResult> {
        let val = self.get_value(&args[0])?;
        println!("{}> {}", self.name, val);
        Ok(())
    }

    fn put_value(&mut self, value: Register, target: &Arg) -> Result<(), ExaResult> {
        // clamp value befor storing
        let value = match value {
            Register::Number(n) => Register::Number(n.clamp(-9999, 9999)),
            Register::Keyword(w) => Register::Keyword(w),
        };
        match target.register().unwrap() {
            RegLabel::X => {
                self.reg_x = value;
                Ok(())
            }
            RegLabel::T => {
                self.reg_t = value;
                Ok(())
            }
            RegLabel::F => Err(ExaResult::Error(Error::InvalidFileAccess)),
            RegLabel::M => {
                self.reg_m = Some(value);
                Err(ExaResult::VMRequest(VMRequest::Tx))
            }
            RegLabel::H(_) => Err(ExaResult::Error(Error::InvalidHWRegisterAccess)),
        }
    }

    fn get_value(&mut self, target: &Arg) -> Result<Register, ExaResult> {
        match target {
            Arg::Register(_) => (),
            _ => return Ok(Register::from_arg(target).unwrap()),
        }
        match target.register().unwrap() {
            RegLabel::X => Ok(self.reg_x.clone()),
            RegLabel::T => Ok(self.reg_t.clone()),
            RegLabel::F => Err(ExaResult::Error(Error::InvalidFileAccess)),
            RegLabel::M => match self.reg_m.take() {
                Some(r) => Ok(r),
                None => Err(ExaResult::VMRequest(VMRequest::Rx)),
            },
            RegLabel::H(_) => Err(ExaResult::Error(Error::InvalidHWRegisterAccess)),
        }
    }

    fn get_number(&mut self, arg: &Arg) -> Result<i16, ExaResult> {
        match arg {
            Arg::Register(_) => {
                let result = self.get_value(arg)?;
                match result {
                    Register::Number(n) => Ok(n),
                    Register::Keyword(_) => Err(ExaResult::Error(Error::MathWithKeywords)),
                }
            }
            Arg::Number(_) => Ok(arg.number().unwrap()),
            _ => Err(ExaResult::Error(Error::MathWithKeywords)),
        }
    }

    pub fn new(name: &str, instr_list: Vec<(Instruction, Option<Vec<Arg>>)>) -> Self {
        Self {
            name: name.to_string(),
            instr_list,
            instr_ptr: 0,
            repl_counter: 0,
            reg_x: Register::Number(0),
            reg_t: Register::Number(0),
            reg_m: None,
        }
    }

    pub fn send_m(&mut self) -> Option<Register> {
        self.instr_ptr += 1;
        Some(self.reg_m.take().unwrap())
    }

    pub fn exec(&mut self) -> Result<(), ExaResult> {
        if self.instr_ptr as usize == self.instr_list.len() {
            return Err(ExaResult::Error(Error::OutOfInstructions));
        }
        let instruction = self.instr_list[self.instr_ptr as usize].clone();
        if instruction.0 == Instruction::Mark {
            self.instr_ptr += 1;
            return self.exec();
        }
        match self.execute_instruction(instruction) {
            Ok(s) => {
                self.instr_ptr += 1;
                Ok(s)
            }
            Err(e) => Err(e),
        }
    }

    fn execute_instruction(
        &mut self,
        (instr, args): (Instruction, Option<Vec<Arg>>),
    ) -> Result<(), ExaResult> {
        let args = match args {
            Some(a) => a,
            None => Vec::with_capacity(0),
        };
        match instr {
            // I/O
            Instruction::Copy => self.copy(args),
            // math
            Instruction::Addi => self.addi(args),
            Instruction::Subi => self.subi(args),
            Instruction::Muli => self.muli(args),
            Instruction::Divi => self.divi(args),
            Instruction::Modi => self.modi(args),
            Instruction::Swiz => self.swiz(args),
            // test
            Instruction::Test => self.test(args),
            // jumps
            Instruction::Jump => self.jump(args),
            Instruction::Tjmp => self.tjmp(args),
            Instruction::Fjmp => self.fjmp(args),
            // lifecycle
            Instruction::Link => self.link(args),
            Instruction::Repl => self.repl(args),
            Instruction::Halt => Self::halt(),
            Instruction::Kill => Self::kill(),
            // misc
            Instruction::Rand => self.rand(args),
            Instruction::Prnt => self.print(args),
            Instruction::Noop => Self::noop(),

            // pseudo-instructions [DO NOT EXECUTE]
            Instruction::Mark => panic!("tried to execute Mark"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Arg {
    Register(RegLabel),
    Number(i16),
    Comp(Comp),
    Keyword(String),
    Label(String),
}

impl Arg {
    pub fn register(&self) -> Result<RegLabel, &str> {
        match self {
            Self::Register(r) => Ok(r.clone()),
            _ => Err("arg is not register label"),
        }
    }

    pub fn number(&self) -> Result<i16, &str> {
        match self {
            Self::Number(n) => Ok(*n),
            _ => Err("arg is not number"),
        }
    }

    pub fn comp(&self) -> Result<Comp, &str> {
        match self {
            Self::Comp(c) => Ok(*c),
            _ => Err("arg is not comparison"),
        }
    }

    pub fn label(&self) -> Result<String, &str> {
        match self {
            Self::Label(l) => Ok(l.to_string()),
            _ => Err("arg is not label"),
        }
    }

    pub fn reg_t() -> Self {
        Arg::Register(RegLabel::T)
    }

    pub fn is_reg_m(&self) -> bool {
        match self {
            Arg::Register(r) => matches!(r, RegLabel::M),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RegLabel {
    X,
    T,
    F,
    M,
    H(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Comp {
    Eq,
    Gt,
    Lt,
    Ge,
    Le,
    Ne,
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

impl From<i16> for Register {
    fn from(value: i16) -> Self {
        Self::Number(value)
    }
}

impl From<String> for Register {
    fn from(value: String) -> Self {
        Self::Keyword(value)
    }
}

impl From<&str> for Register {
    fn from(value: &str) -> Self {
        Self::Keyword(value.to_string())
    }
}

impl Register {
    fn from_arg(arg: &Arg) -> Result<Register, &'static str> {
        match arg {
            Arg::Number(n) => Ok(Register::Number(*n)),
            Arg::Keyword(w) => Ok(Register::Keyword(w.clone())),
            _ => Err("Invalid token type"),
        }
    }
}
