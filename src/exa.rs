use std::{fmt::Display, cmp::Ordering};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum ExaSignal {
    Ok,
    Err(String),
    Repl(Exa),
    Halt,
    Kill,
    Link(i16),
    Tx,
    Rx,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

    Noop,
    Prnt,
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
            Arg::Register(r) => match r {
                RegLabel::M => true,
                _ => false,
            },
            _ => false
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

impl Register {
    fn from_arg(arg: &Arg) -> Result<Register, &'static str> {
        match arg {
            Arg::Number(n) => Ok(Register::Number(*n)),
            Arg::Keyword(w) => Ok(Register::Keyword(w.clone())),
            _ => Err("Invalid token type")
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exa {
    pub name: String,
    instr_list: Vec<(Instruction, Option<Vec<Arg>>)>,
    instr_ptr: u8,
    repl_counter: usize,
    reg_x: Register,
    reg_t: Register,
    pub reg_m: Option<Register>,
}

impl Exa {
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

    pub fn exec(&mut self) -> ExaSignal {
        if self.instr_ptr as usize == self.instr_list.len() { return ExaSignal::Err("Out of Instructions".to_string()); }
        let instruction = self.instr_list[self.instr_ptr as usize].clone();
        if instruction.0 == Instruction::Mark {
            self.instr_ptr += 1;
            return self.exec();
        }
        match self.execute_instruction(instruction) {
            Ok(s) => {
                self.instr_ptr += 1;
                return s;
            },
            Err(e) => return e,
        }
    }

    fn execute_instruction(&mut self, (instr, args): (Instruction, Option<Vec<Arg>>)) -> Result<ExaSignal, ExaSignal> {
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
            Instruction::Prnt => self.print(args),
            Instruction::Noop => Self::noop(),
            _ => Err(ExaSignal::Err("Unknown instruction".to_string())),
        }
    }

    fn link(&mut self, args: Vec<Arg>) -> Result<ExaSignal, ExaSignal> {
        let l = self.get_number(&args[0])?;
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

    fn repl(&mut self, args: Vec<Arg>) -> Result<ExaSignal, ExaSignal> {
        let mut clone = self.clone();
        clone.jump(args)?;
        clone.name.push_str(&format!(":{}", self.repl_counter));
        clone.repl_counter = 0;
        self.repl_counter += 1;
        Ok(ExaSignal::Repl(clone))
    }

    fn addi(&mut self, args: Vec<Arg>) -> Result<ExaSignal, ExaSignal> {
        let num1 = self.get_number(&args[0])?;
        let num2 = self.get_number(&args[1])?;
        self.put_value(Register::Number(num1 + num2), &args[2])?;
        Ok(ExaSignal::Ok)
    }

    fn subi(&mut self, args: Vec<Arg>) -> Result<ExaSignal, ExaSignal> {
        let num1 = self.get_number(&args[0])?;
        let num2 = self.get_number(&args[1])?;
        self.put_value(Register::Number(num1 - num2), &args[2])?;
        Ok(ExaSignal::Ok)
    }

    fn muli(&mut self, args: Vec<Arg>) -> Result<ExaSignal, ExaSignal> {
        let num1 = self.get_number(&args[0])?;
        let num2 = self.get_number(&args[1])?;
        self.put_value(Register::Number(num1 * num2), &args[2])?;
        Ok(ExaSignal::Ok)
    }

    fn divi(&mut self, args: Vec<Arg>) -> Result<ExaSignal, ExaSignal> {
        let num1 = self.get_number(&args[0])?;
        let num2 = self.get_number(&args[1])?;
        self.put_value(Register::Number(num1 / num2), &args[2])?;
        Ok(ExaSignal::Ok)
    }

    fn modi(&mut self, args: Vec<Arg>) -> Result<ExaSignal, ExaSignal> {
        let num1 = self.get_number(&args[0])?;
        let num2 = self.get_number(&args[1])?;
        self.put_value(Register::Number(num1 % num2), &args[2])?;
        Ok(ExaSignal::Ok)
    }

    fn swiz(&mut self, args: Vec<Arg>) -> Result<ExaSignal, ExaSignal> {
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
        Ok(ExaSignal::Ok)
    }

    fn test(&mut self, args: Vec<Arg>) -> Result<ExaSignal, ExaSignal> {
        let v1 = self.get_value(&args[0])?;
        let v2 = self.get_value(&args[2])?;
        let eval;
        match args[1].comp().unwrap() {
            Comp::Eq => eval = v1 == v2,
            Comp::Gt => eval = v1 > v2,
            Comp::Lt => eval = v1 < v2,
            Comp::Ge => eval = v1 >= v2,
            Comp::Le => eval = v1 <= v2,
            Comp::Ne => eval = v1 != v2,
        }
        self.reg_t = Register::Number(eval as i16);
        Ok(ExaSignal::Ok)
    }

    fn jump(&mut self, args: Vec<Arg>) -> Result<ExaSignal, ExaSignal> {
        for x in 0..self.instr_list.len() {
            if self.instr_list[x].0 != Instruction::Mark { continue; }
            if  self.instr_list[x].1.clone().unwrap()[0] == args[0] {
                self.instr_ptr = x as u8;
                return Ok(ExaSignal::Ok);
            }
        }
        Err(ExaSignal::Err("label not found".to_string()))
    }

    fn tjmp(&mut self, args: Vec<Arg>) -> Result<ExaSignal, ExaSignal> {
        if self.get_value(&Arg::reg_t()).unwrap() != Register::Number(0) {
            self.jump(args)
        }
        else { Ok(ExaSignal::Ok) }
    }

    fn fjmp(&mut self, args: Vec<Arg>) -> Result<ExaSignal, ExaSignal> {
        if self.get_value(&Arg::reg_t()).unwrap() == Register::Number(0) {
            self.jump(args)
        }
        else { Ok(ExaSignal::Ok) }
    }

    fn copy(&mut self, args: Vec<Arg>) -> Result<ExaSignal, ExaSignal> {
        let val = self.get_value(&args[0])?;
        self.put_value(val, &args[1])?;
        Ok(ExaSignal::Ok)
    }

    fn print(&mut self, args: Vec<Arg>) -> Result<ExaSignal, ExaSignal> {
        let val = self.get_value(&args[0])?;
        println!("{}> {}", self.name, val);
        Ok(ExaSignal::Ok)
    }

    fn put_value(&mut self, value: Register, target: &Arg) -> Result<(), ExaSignal> {
        let value = match value {
            Register::Number(n) => Register::Number(n.clamp(-9999, 9999)),
            Register::Keyword(w) => Register::Keyword(w),
        };
        match target.register().unwrap() {
            RegLabel::X => *self.gereg_tister_mut(RegLabel::X).unwrap() = value,
            RegLabel::T => *self.gereg_tister_mut(RegLabel::T).unwrap() = value,
            RegLabel::F => panic!("file register not yet implemented"),
            RegLabel::M => {
                self.reg_m = Some(value);
                return Err(ExaSignal::Tx);
            },
            RegLabel::H(_) => panic!("hardware registers not yet implemented"),
        }
        Ok(())
    }

    fn get_value(&mut self, target: &Arg) -> Result<Register, ExaSignal> {
        match target {
            Arg::Register(_) => (),
            _ => return Ok(Register::from_arg(target).unwrap()),
        }
        match target.register().unwrap() {
            RegLabel::X => Ok(self.gereg_tister_ref(RegLabel::X).unwrap().to_owned()),
            RegLabel::T => Ok(self.gereg_tister_ref(RegLabel::T).unwrap().to_owned()),
            RegLabel::F => panic!("file register not yet implemented"),
            RegLabel::M => {
                match self.reg_m.take() {
                    Some(r) => Ok(r),
                    None => Err(ExaSignal::Rx)
                }
            },
            RegLabel::H(_) => panic!("hardware registers not yet implemented"),
        }
    }

    fn gereg_tister_ref(&self, reg: RegLabel) -> Result<&Register, &str> {
        match reg {
            RegLabel::X => Ok(&self.reg_x),
            RegLabel::T => Ok(&self.reg_t),
            _ => Err("Invalid register"),
        }
    }

    fn gereg_tister_mut(&mut self, reg: RegLabel) -> Result<&mut Register, &str> {
        match reg {
            RegLabel::X => Ok(&mut self.reg_x),
            RegLabel::T => Ok(&mut self.reg_t),
            _ => Err("Invalid register"),
        }
    }

    fn get_number<'a>(&'a mut self, arg: &Arg) -> Result<i16, ExaSignal> {
        match arg {
            Arg::Register(_) =>  {
                    let result = self.get_value(arg)?;
                    match result {
                        Register::Number(n) => Ok(n),
                        Register::Keyword(_) => Err(ExaSignal::Err("Not a number".to_string())),
                    }
                },
            Arg::Number(_) => return Ok(arg.number().unwrap()),
            _ => return Err(ExaSignal::Err("Not a number".to_string())),
        }
    }
}
