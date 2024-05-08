use std::{cell::RefCell, collections::HashMap, rc::Rc};

use rand::{rngs::ThreadRng, Rng};

pub use self::config::Config;
pub use self::hw_register::HWRegister;
use crate::exa::{Arg, Comp, Exa, OpCode, RegLabel, Register};
use crate::exa::{File, RegMMode};
pub use hw_register::DebugOutput;

mod config;
mod hw_register;

#[derive(Debug, Clone, Copy)]
enum ExaResult {
    SideEffect(SideEffect),
    Block(Block),
    Error(RuntimeError),
}

impl ExaResult {
    pub fn is_block(&self) -> bool {
        matches!(self, Self::Block(_))
    }
}

#[derive(Debug, Clone, Copy)]
enum SideEffect {
    Repl(u8),
    Link(i16),
    Kill,
    Halt,
}

#[derive(Debug, Clone, Copy)]
enum Block {
    Send,
    Recv,
    Jump,
}

#[derive(Debug, Clone, Copy)]
enum RuntimeError {
    OutOfInstructions,
    FileNotFound,
    NoFileHeld,
    AlreadyHoldingFile,
    InvalidHWRegAccess,
    InvalidFRegAccess,
    InvalidArgument,
    NumericValueRequired,
}

pub struct VM {
    exas: HashMap<usize, RefCell<Exa>>,
    reg_m: RefCell<Option<Register>>,
    rng: RefCell<ThreadRng>,

    files: RefCell<HashMap<i16, File>>,
    hostname: Rc<Box<str>>,
    config: Rc<Config>,
    hw_registers: RefCell<HashMap<Box<str>, Box<dyn HWRegister>>>,
}

impl VM {
    pub fn new(hostname: Rc<Box<str>>, config: Rc<Config>) -> Self {
        let mut hw_registers: HashMap<Box<str>, Box<dyn HWRegister>> = HashMap::new();
        hw_registers.insert(
            "#DBG".into(),
            Box::new(hw_register::DebugOutput {
                prefix: "host::debug>".into(),
            }),
        );
        Self {
            exas: HashMap::with_capacity(config.max_exas),
            reg_m: RefCell::new(None),
            rng: RefCell::new(rand::thread_rng()),
            files: RefCell::new(HashMap::with_capacity(config.max_files)),
            hostname,
            config,
            hw_registers: RefCell::new(hw_registers),
        }
    }

    pub fn step(&mut self) {
        if self.exas.is_empty() {
            return;
        }
        let results = self.exec_all();
        self.apply_side_effects(results);
    }

    pub fn add_exa(&mut self, exa: Exa) {
        self.exas.insert(
            match self.exas.keys().max() {
                Some(n) => n + 1,
                None => 0,
            },
            RefCell::new(exa),
        );
    }

    pub fn add_file(&mut self, f: File) {
        let max = {
            match self.files.borrow().keys().max() {
                Some(n) => n + 1,
                None => 0,
            }
        };
        let mut files = self.files.borrow_mut();
        files.insert(max, f);
    }

    fn exec_all(&mut self) -> Vec<(usize, ExaResult)> {
        let mut results = Vec::with_capacity(self.exas.len());
        for (i, exa) in self.exas.iter() {
            if let Err(res) = self.exec(exa) {
                results.push((*i, res));
            }
        }
        results
    }

    fn apply_side_effects(&mut self, results: Vec<(usize, ExaResult)>) {
        for (k, res) in results {
            match res {
                ExaResult::SideEffect(se) => match se {
                    SideEffect::Repl(j) => {
                        let (key, val) = self.generate_clone(&k, j);
                        self.exas.insert(key, val);
                    }
                    SideEffect::Kill => {
                        for k2 in self.exas.keys() {
                            if k2 != &k {
                                self.exas.remove(&k);
                                break;
                            }
                        }
                    }
                    SideEffect::Link(_) => {
                        self.exas.remove(&k);
                    }
                    SideEffect::Halt => {
                        self.exas.remove(&k);
                    }
                },
                ExaResult::Block(it) => match it {
                    Block::Recv => {}
                    Block::Send => {}
                    Block::Jump => {}
                },
                ExaResult::Error(e) => {
                    let name = self.exas.remove(&k).unwrap().borrow().name.clone();
                    println!("{}| {:?}", name, e);
                }
            }
        }
    }

    fn generate_clone(&mut self, k: &usize, j: u8) -> (usize, RefCell<Exa>) {
        let mut clone = self.exas.get(k).unwrap().borrow().clone();
        clone.instr_ptr = j + 1;
        clone.name.push_str(&format!(":{}", clone.repl_counter));
        clone.repl_counter = 0;
        (self.exas.keys().max().unwrap() + 1, RefCell::new(clone))
    }

    fn exec(&self, exa: &RefCell<Exa>) -> Result<(), ExaResult> {
        let instr = {
            let eb = exa.borrow();
            if eb.instr_ptr as usize == eb.instr_list.len() {
                return Err(ExaResult::Error(RuntimeError::OutOfInstructions));
            }
            eb.instr_list[eb.instr_ptr as usize].clone()
        };

        // skip MARKs
        if instr.0 == OpCode::Mark {
            exa.borrow_mut().instr_ptr += 1;
            return self.exec(exa);
        }

        let res: Result<(), ExaResult> = match instr.0 {
            OpCode::Copy => self.copy(exa, instr.two_args()),
            OpCode::Void => self.void(exa, instr.one_arg()),

            OpCode::Addi => self.addi(exa, instr.three_args()),
            OpCode::Subi => self.subi(exa, instr.three_args()),
            OpCode::Muli => self.muli(exa, instr.three_args()),
            OpCode::Divi => self.divi(exa, instr.three_args()),
            OpCode::Modi => self.modi(exa, instr.three_args()),
            OpCode::Swiz => self.swiz(exa, instr.three_args()),
            OpCode::Rand => self.rand(exa, instr.three_args()),
            OpCode::Mode => Self::mode(exa),

            OpCode::Test => self.test(exa, instr.three_args()),
            OpCode::TestMrd => self.test_mrd(exa),
            OpCode::TestEof => Self::test_eof(exa),

            OpCode::Jump => Self::jump(exa, instr.one_arg()),
            OpCode::Tjmp => Self::tjmp(exa, instr.one_arg()),
            OpCode::Fjmp => Self::fjmp(exa, instr.one_arg()),

            OpCode::Make => self.make(exa),
            OpCode::Grab => self.grab(exa, instr.one_arg()),
            OpCode::File => self.file(exa, instr.one_arg()),
            OpCode::Seek => self.seek(exa, instr.one_arg()),
            OpCode::Drop => self.drop(exa),
            OpCode::Wipe => Self::wipe(exa),

            OpCode::Link => Err(ExaResult::SideEffect(SideEffect::Link(
                instr.one_arg().number().unwrap(),
            ))),
            OpCode::Repl => Self::repl(exa, instr.one_arg()),
            OpCode::Halt => Err(ExaResult::SideEffect(SideEffect::Halt)),
            OpCode::Kill => Err(ExaResult::SideEffect(SideEffect::Kill)),

            OpCode::Host => self.host(exa, instr.one_arg()),
            OpCode::Noop => Ok(()),

            OpCode::Mark => unreachable!(),

            OpCode::Prnt => self.prnt(exa, instr.one_arg()),
        };

        if let Err(e) = res {
            if e.is_block() {
                return Err(e);
            }
        }
        exa.borrow_mut().instr_ptr += 1;
        res
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new(Rc::new("Rhizome".into()), Rc::new(Config::default()))
    }
}

// -----------------------------------------------------------
//                   Instruction Execution
// -----------------------------------------------------------

impl VM {
    fn copy(&self, exa: &RefCell<Exa>, (value, target): (Arg, Arg)) -> Result<(), ExaResult> {
        let val = self.get_value(exa, value)?;
        self.put_value(exa, val, target.reg_label().unwrap())?;
        Ok(())
    }

    fn void(&self, exa: &RefCell<Exa>, target: Arg) -> Result<(), ExaResult> {
        match target.reg_label().unwrap() {
            RegLabel::X => exa.borrow_mut().reg_x = Register::Number(0),
            RegLabel::T => exa.borrow_mut().reg_t = Register::Number(0),
            RegLabel::F => self.put_value(
                exa,
                Register::Keyword("".to_string().into_boxed_str()),
                RegLabel::F,
            )?,
            RegLabel::M => match self.reg_m.borrow_mut().take() {
                Some(_) => return Ok(()),
                None => return Err(ExaResult::Block(Block::Recv)),
            },
            RegLabel::H(_) => return Err(ExaResult::Error(RuntimeError::InvalidHWRegAccess)),
        }
        Ok(())
    }

    fn addi(
        &self,
        exa: &RefCell<Exa>,
        (num1, num2, target): (Arg, Arg, Arg),
    ) -> Result<(), ExaResult> {
        let num1 = self.get_number(exa, num1)?;
        let num2 = self.get_number(exa, num2)?;
        self.put_value(
            exa,
            Register::Number(num1 + num2),
            target.reg_label().unwrap(),
        )?;
        Ok(())
    }

    fn subi(
        &self,
        exa: &RefCell<Exa>,
        (num1, num2, target): (Arg, Arg, Arg),
    ) -> Result<(), ExaResult> {
        let num1 = self.get_number(exa, num1)?;
        let num2 = self.get_number(exa, num2)?;
        self.put_value(
            exa,
            Register::Number(num1 - num2),
            target.reg_label().unwrap(),
        )?;
        Ok(())
    }

    fn muli(
        &self,
        exa: &RefCell<Exa>,
        (num1, num2, target): (Arg, Arg, Arg),
    ) -> Result<(), ExaResult> {
        let num1 = self.get_number(exa, num1)?;
        let num2 = self.get_number(exa, num2)?;
        self.put_value(
            exa,
            Register::Number(i16::saturating_mul(num1, num2)),
            target.reg_label().unwrap(),
        )?;
        Ok(())
    }

    fn divi(
        &self,
        exa: &RefCell<Exa>,
        (num1, num2, target): (Arg, Arg, Arg),
    ) -> Result<(), ExaResult> {
        let num1 = self.get_number(exa, num1)?;
        let num2 = self.get_number(exa, num2)?;
        self.put_value(
            exa,
            Register::Number(num1 / num2),
            target.reg_label().unwrap(),
        )?;
        Ok(())
    }

    fn modi(
        &self,
        exa: &RefCell<Exa>,
        (num1, num2, target): (Arg, Arg, Arg),
    ) -> Result<(), ExaResult> {
        let num1 = self.get_number(exa, num1)?;
        let num2 = self.get_number(exa, num2)?;
        self.put_value(
            exa,
            Register::Number(num1 % num2),
            target.reg_label().unwrap(),
        )?;
        Ok(())
    }

    fn swiz(
        &self,
        exa: &RefCell<Exa>,
        (num1, num2, target): (Arg, Arg, Arg),
    ) -> Result<(), ExaResult> {
        let num1 = self.get_number(exa, num1)?;
        let num2 = self.get_number(exa, num2)?;
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
        self.put_value(exa, Register::Number(result), target.reg_label().unwrap())?;
        Ok(())
    }

    fn rand(
        &self,
        exa: &RefCell<Exa>,
        (num1, num2, target): (Arg, Arg, Arg),
    ) -> Result<(), ExaResult> {
        let num1 = self.get_number(exa, num1)?;
        let num2 = self.get_number(exa, num2)?;
        let mut rng = self.rng.borrow_mut();
        if num1 < num2 {
            self.put_value(
                exa,
                Register::Number(rng.gen_range(num1..=num2)),
                target.reg_label().unwrap(),
            )?;
        } else {
            self.put_value(
                exa,
                Register::Number(rng.gen_range(num2..=num1)),
                target.reg_label().unwrap(),
            )?;
        }
        Ok(())
    }

    fn mode(exa: &RefCell<Exa>) -> Result<(), ExaResult> {
        let mut exa = exa.borrow_mut();
        exa.mode = match exa.mode {
            RegMMode::Global => RegMMode::Local,
            RegMMode::Local => RegMMode::Global,
        };
        Ok(())
    }

    fn test(
        &self,
        exa: &RefCell<Exa>,
        (arg1, comp, arg2): (Arg, Arg, Arg),
    ) -> Result<(), ExaResult> {
        let v1 = self.get_value(exa, arg1)?;
        let v2 = self.get_value(exa, arg2)?;
        let eval = match comp.comp().unwrap() {
            Comp::Eq => v1 == v2,
            Comp::Gt => v1 > v2,
            Comp::Lt => v1 < v2,
            Comp::Ge => v1 >= v2,
            Comp::Le => v1 <= v2,
            Comp::Ne => v1 != v2,
        };
        exa.borrow_mut().reg_t = Register::Number(eval as i16);
        Ok(())
    }

    fn test_eof(exa: &RefCell<Exa>) -> Result<(), ExaResult> {
        let eof = match exa.borrow().reg_f.as_ref() {
            Some(f) => f.1.is_eof(),
            None => return Err(ExaResult::Error(RuntimeError::NoFileHeld)),
        } as i16;
        {
            exa.borrow_mut().reg_t = Register::Number(eof);
        }
        Ok(())
    }

    fn test_mrd(&self, exa: &RefCell<Exa>) -> Result<(), ExaResult> {
        {
            exa.borrow_mut().reg_t = Register::Number(self.reg_m.borrow().is_some() as i16);
        }
        Ok(())
    }

    fn tjmp(exa: &RefCell<Exa>, target: Arg) -> Result<(), ExaResult> {
        if exa.borrow().reg_t == Register::Number(0) {
            return Ok(());
        }
        Self::jump(exa, target)
    }

    fn fjmp(exa: &RefCell<Exa>, target: Arg) -> Result<(), ExaResult> {
        if exa.borrow().reg_t != Register::Number(0) {
            return Ok(());
        }
        Self::jump(exa, target)
    }

    fn jump(exa: &RefCell<Exa>, target: Arg) -> Result<(), ExaResult> {
        exa.borrow_mut().instr_ptr = match target.jump_index() {
            Ok(n) => n as u8,
            Err(_) => return Err(ExaResult::Error(RuntimeError::InvalidArgument)),
        };
        Err(ExaResult::Block(Block::Jump))
    }

    fn make(&self, exa: &RefCell<Exa>) -> Result<(), ExaResult> {
        if exa.borrow().reg_f.is_some() {
            return Err(ExaResult::Error(RuntimeError::AlreadyHoldingFile));
        }
        exa.borrow_mut().reg_f = Some((
            self.files
                .borrow()
                .keys()
                .max()
                .unwrap_or(&300i16)
                .to_owned(),
            File::default(),
        ));
        Ok(())
    }

    fn grab(&self, exa: &RefCell<Exa>, target: Arg) -> Result<(), ExaResult> {
        match self
            .files
            .borrow_mut()
            .remove_entry(&self.get_number(exa, target)?)
        {
            Some(t) => {
                let mut e = exa.borrow_mut();
                match e.reg_f {
                    None => {
                        e.reg_f = Some(t);
                        Ok(())
                    }
                    Some(_) => Err(ExaResult::Error(RuntimeError::AlreadyHoldingFile)),
                }
            }
            None => Err(ExaResult::Error(RuntimeError::FileNotFound)),
        }
    }

    fn file(&self, exa: &RefCell<Exa>, arg1: Arg) -> Result<(), ExaResult> {
        let f = match exa.borrow().reg_f.as_ref() {
            Some(f) => Ok(f.0),
            None => Err(ExaResult::Error(RuntimeError::NoFileHeld)),
        }?;
        self.put_value(exa, Register::Number(f), arg1.reg_label().unwrap())
    }

    fn seek(&self, exa: &RefCell<Exa>, arg1: Arg) -> Result<(), ExaResult> {
        let n = self.get_number(exa, arg1)?;
        match exa.borrow_mut().reg_f.as_mut() {
            Some(f) => {
                f.1.seek(n);
                Ok(())
            }
            None => Err(ExaResult::Error(RuntimeError::NoFileHeld)),
        }
    }

    fn drop(&self, exa: &RefCell<Exa>) -> Result<(), ExaResult> {
        if let Some(f) = exa.borrow_mut().reg_f.take() {
            self.files.borrow_mut().insert(f.0, f.1);
            return Ok(());
        }
        Err(ExaResult::Error(RuntimeError::NoFileHeld))
    }

    fn wipe(exa: &RefCell<Exa>) -> Result<(), ExaResult> {
        if exa.borrow_mut().reg_f.take().is_some() {
            return Ok(());
        }
        Err(ExaResult::Error(RuntimeError::NoFileHeld))
    }

    fn repl(exa: &RefCell<Exa>, target: Arg) -> Result<(), ExaResult> {
        let ptr = exa.borrow().instr_ptr;
        Self::jump(exa, target)?;
        let traget_ptr = { exa.borrow().instr_ptr };
        exa.borrow_mut().instr_ptr = ptr;
        Err(ExaResult::SideEffect(SideEffect::Repl(traget_ptr)))
    }

    fn host(&self, exa: &RefCell<Exa>, target: Arg) -> Result<(), ExaResult> {
        self.put_value(
            exa,
            Register::Keyword(self.hostname.to_string().into_boxed_str()),
            target.reg_label().unwrap(),
        )?;
        Ok(())
    }

    fn prnt(&self, exa: &RefCell<Exa>, target: Arg) -> Result<(), ExaResult> {
        let name = { exa.borrow().name.clone() };
        println!("{}> {}", name, self.get_value(exa, target)?);
        Ok(())
    }

    fn get_number(&self, exa: &RefCell<Exa>, target: Arg) -> Result<i16, ExaResult> {
        match self.get_value(exa, target)? {
            Register::Number(n) => Ok(n),
            Register::Keyword(_) => Err(ExaResult::Error(RuntimeError::NumericValueRequired)),
        }
    }

    fn get_value(&self, exa: &RefCell<Exa>, target: Arg) -> Result<Register, ExaResult> {
        match target {
            Arg::Number(n) => Ok(Register::Number(n)),
            Arg::Keyword(k) => Ok(Register::Keyword(k)),
            Arg::RegLabel(r) => match r {
                RegLabel::X => Ok(exa.borrow().reg_x.clone()),
                RegLabel::T => Ok(exa.borrow().reg_t.clone()),
                RegLabel::F => {
                    if let Some(f_ref) = exa.borrow_mut().reg_f.as_mut() {
                        match f_ref.1.read() {
                            Some(r) => Ok(r),
                            None => Err(ExaResult::Error(RuntimeError::InvalidFRegAccess)),
                        }
                    } else {
                        Err(ExaResult::Error(RuntimeError::NoFileHeld))
                    }
                }
                RegLabel::M => match self.reg_m.take() {
                    Some(r) => Ok(r),
                    None => Err(ExaResult::Block(Block::Recv)),
                },
                RegLabel::H(reg) => match self.hw_registers.borrow().get(&reg) {
                    Some(hwr) => hwr.read(),
                    None => Err(ExaResult::Error(RuntimeError::InvalidHWRegAccess)),
                },
            },
            _ => Err(ExaResult::Error(RuntimeError::InvalidArgument)),
        }
    }

    fn put_value(
        &self,
        exa: &RefCell<Exa>,
        value: Register,
        target: RegLabel,
    ) -> Result<(), ExaResult> {
        let value = match value {
            Register::Number(n) => Register::Number(n.clamp(-9999, 9999)),
            Register::Keyword(w) => Register::Keyword(w),
        };
        match target {
            RegLabel::X => {
                exa.borrow_mut().reg_x = value;
                Ok(())
            }
            RegLabel::T => {
                exa.borrow_mut().reg_t = value;
                Ok(())
            }
            RegLabel::F => {
                if let Some(f_ref) = exa.borrow_mut().reg_f.as_mut() {
                    f_ref.1.write(value);
                    Ok(())
                } else {
                    Err(ExaResult::Error(RuntimeError::InvalidFRegAccess))
                }
            }
            RegLabel::M => {
                let mut reg_m = self.reg_m.borrow_mut();
                if reg_m.is_some() {
                    Err(ExaResult::Block(Block::Send))
                } else {
                    *reg_m = Some(value);
                    Ok(())
                }
            }
            RegLabel::H(reg) => match self.hw_registers.borrow_mut().get_mut(&reg) {
                Some(hwr) => hwr.write(value),
                None => Err(ExaResult::Error(RuntimeError::InvalidHWRegAccess)),
            },
        }
    }
}
