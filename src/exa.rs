mod lexar;

use std::fmt::Display;
use once_cell::sync::Lazy;
use regex::RegexSet;
use lexar::*;

#[derive(Debug, Clone)]
pub struct Exa {
    instr_list: Vec<String>,
    instr_ptr: u8,
    reg_x: Register,
    reg_t: Register,
}

#[derive(Debug, Clone)]
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

impl Register {
    fn get_number(&self) -> Result<i16, String> {
        match self {
            Self::Number(n) => Ok(*n),
            _ => Err("Not a Number".to_string()),
        }
    }

    fn get_keyword(&self) -> Result<String, String> {
        match self {
            Self::Keyword(w) => Ok(w.to_owned()),
            _ => Err("Not a Keyword".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Arg {
    Value(Register),
    RegisterLabel(char),
    Comparison(String),
}

impl Arg {
    fn from_keyword(s: &String) -> Arg {
        let mut a = s.to_string();
        a.remove(0);
        a.remove(a.len() - 1);
        Arg::Value(Register::Keyword(a))
    }

    fn from_label(s: &String) -> Arg {
        Arg::Value(Register::Keyword(s.to_string()))
    }

    fn from_number(s: &String) -> Arg {
        Arg::Value(Register::Number(s.parse::<i16>().unwrap()))
    }

    fn from_register(s: &String) -> Arg {
        let mut r = s.to_string();
        return Arg::RegisterLabel(r.pop().unwrap());
    }

    fn from_comparison(s: &String) -> Arg {
        Arg::Comparison(s.to_string())
    }
}

impl Exa {
    pub fn new(code: Vec<String>) -> Result<Exa, Vec<String>> {
        match compile(code) {
            Ok(instr_list) => Ok(Exa {
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
        let tokens: Vec<String> = Self::tokenize(&self.instr_list[self.instr_ptr as usize]);
        self.instr_ptr += 1;
        if tokens[0] == "mark".to_string() { return Ok(()); }
        let args: Vec<Arg> = Self::parse_args(&self, &tokens[1..]).unwrap();
        self.execute_instruction(&tokens[0], args)
    }

    fn execute_instruction(&mut self, instr: &String, args: Vec<Arg>) -> Result<(), &str> {
        match &instr[..] {
            // I/O
            "copy" => self.copy(&args[0], &args[1]),
            // math
            "addi" => self.addi(&args[0], &args[1], &args[2]),
            "subi" => self.subi(&args[0], &args[1], &args[2]),
            "muli" => self.muli(&args[0], &args[1], &args[2]),
            "divi" => self.divi(&args[0], &args[1], &args[2]),
            "modi" => self.modi(&args[0], &args[1], &args[2]),
            // test
            "test" => self.test(&args[0], &args[1], &args[2]),
            // jumps
            "jump" => self.jump(&args[0]),
            "tjmp" => self.tjmp(&args[0]),
            "fjmp" => self.fjmp(&args[0]),
            // misc
            "prnt" => self.print(&args[0]),
            _ => Err("Unknown instruction"),
        }
    }

    fn parse_args(&self, tokens: &[String]) -> Result<Vec<Arg>, &str> {
        let mut args: Vec<Arg> = Vec::with_capacity(3);
        static RS: Lazy<RegexSet> = Lazy::new(|| RegexSet::new([
            r"'*'",          // Keyword
            r"[A-Z]+[0-9]*", // Label
            r"[0-9]+",       // Number 
            r"[xtfm]{1}",    // Register
            r"[=!><]{1,2}",  // Comparison
        ]).unwrap());
        for t in tokens {
            match RS.matches(t).into_iter().nth(0).unwrap() {
                0 => args.push(Arg::from_keyword(t)),
                1 => args.push(Arg::from_label(t)),
                2 => args.push(Arg::from_number(t)),
                3 => args.push(Arg::from_register(t)),
                4 => args.push(Arg::from_comparison(t)),
                _ => return Err("Unknown Arg type"),
            }
        }
        Ok(args)
    }

    fn tokenize(instr: &String) -> Vec<String> {
        let mut tokens: Vec<String> = vec![ instr[..4].to_string() ];
        if instr.len() > 5 {
            let arg_slice = &instr[5..];
            let mut t_begin: usize = 0;
            let mut mid_word: bool = false;
            for x in 0..arg_slice.len() {
                match arg_slice.chars().nth(x).unwrap() {
                    '\'' => {
                        mid_word = !mid_word;
                        if !mid_word {
                            tokens.push(arg_slice[t_begin..(x + 1)].to_string());
                            t_begin = x + 1;
                        }
                    },
                    ' ' => {
                        if !mid_word {
                            tokens.push(arg_slice[t_begin..x].to_string());
                            t_begin = x + 1;
                        }
                    },
                    _ => {},
                }
            }
            if arg_slice.len() > t_begin {
                tokens.push(arg_slice[t_begin..arg_slice.len()].to_string());
            }
        }
        tokens
    }

    fn addi(&mut self, op1: &Arg, op2: &Arg, target: &Arg) -> Result<(), &str> {
        let num1 = self.get_number(op1).unwrap();
        let num2 = self.get_number(op2).unwrap();
        *self.get_register_mut(target).unwrap() = Register::Number(num1 + num2);
        Ok(())
    }

    fn subi(&mut self, op1: &Arg, op2: &Arg, target: &Arg) -> Result<(), &str> {
        let num1 = self.get_number(op1).unwrap();
        let num2 = self.get_number(op2).unwrap();
        *self.get_register_mut(target).unwrap() = Register::Number(num1 - num2);
        Ok(())
    }

    fn muli(&mut self, op1: &Arg, op2: &Arg, target: &Arg) -> Result<(), &str> {
        let num1 = self.get_number(op1).unwrap();
        let num2 = self.get_number(op2).unwrap();
        *self.get_register_mut(target).unwrap() = Register::Number(num1 * num2);
        Ok(())
    }

    fn divi(&mut self, op1: &Arg, op2: &Arg, target: &Arg) -> Result<(), &str> {
        let num1 = self.get_number(op1).unwrap();
        let num2 = self.get_number(op2).unwrap();
        *self.get_register_mut(target).unwrap() = Register::Number(num1 / num2);
        Ok(())
    }

    fn modi(&mut self, op1: &Arg, op2: &Arg, target: &Arg) -> Result<(), &str> {
        let num1 = self.get_number(op1).unwrap();
        let num2 = self.get_number(op2).unwrap();
        *self.get_register_mut(target).unwrap() = Register::Number(num1 % num2);
        Ok(())
    }

    fn test(&mut self, op1: &Arg, comp: &Arg, op2: &Arg) -> Result<(), &str> {
        // TODO: handle non numbers
        let eval;
        let num1 = self.get_number(op1).unwrap();
        let num2 = self.get_number(op2).unwrap();
        match comp {
            Arg::Comparison(c) => {
                match &c[..] {
                    "=" => eval = num1 == num2,
                    "!=" => eval = num1 != num2,
                    ">=" => eval = num1 >= num2,
                    "<=" => eval = num1 <= num2,
                    ">" => eval = num1 > num2,
                    "<" => eval = num1 < num2,
                    _ => return Err("Invalid Comparison"),
                }
            },
            _ => return Err("Arg[2] is not a Comparison")
        }
        if eval { self.reg_t = Register::Number(1); }
        else { self.reg_t = Register::Number(0); }
        Ok(())
    }

    fn jump(&mut self, arg: &Arg) -> Result<(), &str> {
        let label = self.get_keyword(arg).unwrap();
        for x in 0..self.instr_list.len() {
            let tokens = Self::tokenize(&self.instr_list[x]);
            if tokens[0] == "mark" && tokens[1] == label {
                self.instr_ptr = x as u8;
                return Ok(());
            }
        }
        Err("Label not found")
    }

    fn tjmp(&mut self, arg: &Arg) -> Result<(), &str> {
        if let Ok(t) = self.reg_t.get_number() {
            if t != 0 { self.jump(arg) }
            else { Ok(()) }
        }
        else { self.jump(arg) }
    }

    fn fjmp(&mut self, arg: &Arg) -> Result<(), &str> {
        if let Ok(t) = self.reg_t.get_number() {
            if t == 0 { return self.jump(arg); }
        }
        Ok(())
    }

    fn copy(&mut self, value: &Arg, target: &Arg) -> Result<(), &str> {
        let val: Register;
        match value {
            Arg::RegisterLabel(_) => val = self.get_register_shared(value).unwrap().to_owned(),
            Arg::Value(v) => val = v.to_owned(),
            Arg::Comparison(_) => return Err("Found Comparison instead of Value"),
        }
        *self.get_register_mut(target).unwrap() = val;
        Ok(())
    }

    fn print(&self, arg: &Arg) -> Result<(), &str>{
        match arg {
            Arg::RegisterLabel(_) => Ok(
                println!(">{}", *self.get_register_shared(arg).unwrap())
                ),
            Arg::Value(v) => {
                match v {
                    Register::Number(n) => Ok(println!(">{}", n)),
                    Register::Keyword(w) => Ok(println!(">{}", w)),
                }
            },
            Arg::Comparison(_) => Err("Found Comparison instead of Value"),
        }
    }

    fn get_register_shared(&self, arg: &Arg) -> Result<&Register, &str> {
        match arg {
            Arg::RegisterLabel(r) => {
                match r {
                    'x' => Ok(&self.reg_x),
                    't' => Ok(&self.reg_t),
                    _ => Err("Invalid source register"),
                }
            },
            _ => Err("Not a Register"),
        }
    }

    fn get_register_mut(&mut self, arg: &Arg) -> Result<&mut Register, &str> {
        match arg {
            Arg::RegisterLabel(r) => {
                match r {
                    'x' => Ok(&mut self.reg_x),
                    't' => Ok(&mut self.reg_t),
                    _ => Err("Invalid source register"),
                }
            },
            _ => Err("Not a Register"),
        }
    }

    fn get_keyword(&self, arg: &Arg) -> Result<String, String> {
        let val: Register;
        match arg {
            Arg::RegisterLabel(r) => {
                match r {
                    'x' => val = self.reg_x.to_owned(),
                    't' => val = self.reg_t.to_owned(),
                    _ => return Err("Invalid source register".to_string())
                };
            },
            Arg::Value(v) => val = v.to_owned(),
            Arg::Comparison(_) => return Err("Found Comparison instead of Value".to_string()),
        };
        val.get_keyword()
    }

    fn get_number(&self, arg: &Arg) -> Result<i16, String> {
        let val: Register;
        match arg {
            Arg::RegisterLabel(r) => {
                match r {
                    'x' => val = self.reg_x.to_owned(),
                    't' => val = self.reg_t.to_owned(),
                    _ => return Err("Invalid source register".to_string())
                };
            },
            Arg::Value(v) => val = v.to_owned(),
            Arg::Comparison(_) => return Err("Found Comparison instead of Value".to_string()),
        };
        val.get_number()
    }
}
