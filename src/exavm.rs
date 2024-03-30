use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use rand::{rngs::ThreadRng, Rng};

use crate::exa::{Arg, Comp, Exa, Instruction, RegLabel, Register};

enum ExaResult {
    SideEffect(SideEffect),
    Error(RuntimeError),
}

enum SideEffect {
    Halt,
    Kill,
    Send,
    Recv,
    Repl(u8),
    Link(i16),
    Error(RuntimeError),
}

enum RuntimeError {
    OutOfInstructions,
    UnsupportedInstruction,
    InvalidFileAccess,
    InvalidHWRegAccess,
    InvalidArgument,
    MathWithKeywords,
    LabelNotFound,
}

#[derive(Debug)]
pub struct VM {
    exas: HashMap<usize, Rc<RefCell<Exa>>>,
    send: HashSet<usize>,
    recv: HashSet<usize>,

    reg_m: RefCell<Option<Register>>,
    rng: RefCell<ThreadRng>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            exas: HashMap::new(),
            send: HashSet::new(),
            recv: HashSet::new(),
            reg_m: RefCell::new(None),
            rng: RefCell::new(rand::thread_rng()),
        }
    }

    pub fn step(&mut self) {
        let side_effects = self.exec_all();
        self.apply_side_effects(side_effects);
        // self.handle_reg_m();
    }

    pub fn add_exa(&mut self, exa: Exa) {
        self.exas.insert(
            match self.exas.keys().max() {
                Some(n) => n + 1,
                None => 0,
            },
            Rc::new(RefCell::new(exa)),
        );
    }

    fn exec_all(&mut self) -> Vec<(usize, SideEffect)> {
        let mut side_effects = Vec::with_capacity(self.exas.len());
        for (i, exa) in self
            .exas
            .iter()
            .filter(|(k, _)| !(self.recv.contains(k) || self.send.contains(k)))
        {
            if let Err(res) = self.exec(exa.clone()) {
                match res {
                    ExaResult::SideEffect(se) => {
                        side_effects.push((*i, se));
                    }
                    ExaResult::Error(e) => {
                        side_effects.push((*i, SideEffect::Error(e)));
                    }
                }
            }
        }
        side_effects
    }

    fn apply_side_effects(&mut self, side_effects: Vec<(usize, SideEffect)>) {
        for (k, effect) in side_effects {
            match effect {
                SideEffect::Recv => {
                    self.recv.insert(k);
                }
                SideEffect::Send => {
                    self.send.insert(k);
                }
                SideEffect::Repl(j) => {
                    let (key, val) = self.add_clone(&k, j);
                    self.exas.insert(key, val);
                }
                SideEffect::Halt | SideEffect::Error(_) => {
                    self.exas.remove(&k);
                }
                SideEffect::Link(_) => {
                    self.exas.remove(&k);
                }
                SideEffect::Kill => {
                    for k2 in self.exas.keys() {
                        if k2 != &k {
                            self.exas.remove(&k);
                            break;
                        }
                    }
                }
            }
        }
    }

    // fn handle_reg_m(&mut self) {
    //     if self.send.is_empty() || self.recv.is_empty() {
    //         return;
    //     }
    // }

    fn add_clone(&mut self, k: &usize, j: u8) -> (usize, Rc<RefCell<Exa>>) {
        let mut clone = self.exas.get(k).unwrap().borrow().clone();
        clone.instr_ptr = j + 1;
        clone.name.push_str(&format!(":{}", clone.repl_counter));
        clone.repl_counter = 0;
        (
            self.exas.keys().max().unwrap() + 1,
            Rc::new(RefCell::new(clone)),
        )
    }

    fn exec(&self, exa: Rc<RefCell<Exa>>) -> Result<(), ExaResult> {
        let (instr, arg1, arg2, arg3) = {
            let e = exa.borrow();
            if e.instr_ptr as usize == e.instr_list.len() {
                return Err(ExaResult::Error(RuntimeError::OutOfInstructions));
            }
            e.instr_list[e.instr_ptr as usize].clone()
        };

        // skip MARKs
        if instr == Instruction::Mark {
            {
                exa.borrow_mut().instr_ptr += 1;
            }
            return self.exec(exa);
        }

        match instr {
            Instruction::Copy => self.copy(&exa, arg1.unwrap(), arg2.unwrap()),

            Instruction::Addi => self.addi(&exa, arg1.unwrap(), arg2.unwrap(), arg3.unwrap()),
            Instruction::Subi => self.subi(&exa, arg1.unwrap(), arg2.unwrap(), arg3.unwrap()),
            Instruction::Muli => self.muli(&exa, arg1.unwrap(), arg2.unwrap(), arg3.unwrap()),
            Instruction::Divi => self.divi(&exa, arg1.unwrap(), arg2.unwrap(), arg3.unwrap()),
            Instruction::Modi => self.modi(&exa, arg1.unwrap(), arg2.unwrap(), arg3.unwrap()),
            Instruction::Swiz => self.swiz(&exa, arg1.unwrap(), arg2.unwrap(), arg3.unwrap()),
            Instruction::Rand => self.rand(&exa, arg1.unwrap(), arg2.unwrap(), arg3.unwrap()),

            Instruction::Test => self.test(&exa, arg1.unwrap(), arg2.unwrap(), arg3.unwrap()),

            Instruction::Jump => self.jump(&exa, arg1.unwrap()),
            Instruction::Tjmp => self.tjmp(&exa, arg1.unwrap()),
            Instruction::Fjmp => self.fjmp(&exa, arg1.unwrap()),

            Instruction::Link => Err(ExaResult::SideEffect(SideEffect::Link(
                arg1.unwrap().number().unwrap(),
            ))),
            Instruction::Repl => self.repl(&exa, arg1.unwrap()),
            Instruction::Halt => Err(ExaResult::SideEffect(SideEffect::Halt)),
            Instruction::Kill => Err(ExaResult::SideEffect(SideEffect::Kill)),

            Instruction::Host => self.host(&exa, arg1.unwrap()),
            Instruction::Noop => Ok(()),

            Instruction::Mark => Err(ExaResult::Error(RuntimeError::UnsupportedInstruction)),

            Instruction::Prnt => self.prnt(&exa, arg1.unwrap()),
        }
    }

    fn copy(&self, exa: &Rc<RefCell<Exa>>, value: Arg, target: Arg) -> Result<(), ExaResult> {
        let val = self.get_value(exa, value)?;
        self.put_value(exa, val, target.reg_label().unwrap())?;
        Self::inc_ptr(exa);
        Ok(())
    }

    fn addi(
        &self,
        exa: &Rc<RefCell<Exa>>,
        num1: Arg,
        num2: Arg,
        target: Arg,
    ) -> Result<(), ExaResult> {
        let num1 = self.get_number(exa, num1)?;
        let num2 = self.get_number(exa, num2)?;
        self.put_value(
            exa,
            Register::Number(num1 + num2),
            target.reg_label().unwrap(),
        )?;
        Self::inc_ptr(exa);
        Ok(())
    }

    fn subi(
        &self,
        exa: &Rc<RefCell<Exa>>,
        num1: Arg,
        num2: Arg,
        target: Arg,
    ) -> Result<(), ExaResult> {
        let num1 = self.get_number(exa, num1)?;
        let num2 = self.get_number(exa, num2)?;
        self.put_value(
            exa,
            Register::Number(num1 - num2),
            target.reg_label().unwrap(),
        )?;
        Self::inc_ptr(exa);
        Ok(())
    }

    fn muli(
        &self,
        exa: &Rc<RefCell<Exa>>,
        num1: Arg,
        num2: Arg,
        target: Arg,
    ) -> Result<(), ExaResult> {
        let num1 = self.get_number(exa, num1)?;
        let num2 = self.get_number(exa, num2)?;
        self.put_value(
            exa,
            Register::Number(num1 * num2),
            target.reg_label().unwrap(),
        )?;
        Self::inc_ptr(exa);
        Ok(())
    }

    fn divi(
        &self,
        exa: &Rc<RefCell<Exa>>,
        num1: Arg,
        num2: Arg,
        target: Arg,
    ) -> Result<(), ExaResult> {
        let num1 = self.get_number(exa, num1)?;
        let num2 = self.get_number(exa, num2)?;
        self.put_value(
            exa,
            Register::Number(num1 / num2),
            target.reg_label().unwrap(),
        )?;
        Self::inc_ptr(exa);
        Ok(())
    }

    fn modi(
        &self,
        exa: &Rc<RefCell<Exa>>,
        num1: Arg,
        num2: Arg,
        target: Arg,
    ) -> Result<(), ExaResult> {
        let num1 = self.get_number(exa, num1)?;
        let num2 = self.get_number(exa, num2)?;
        self.put_value(
            exa,
            Register::Number(num1 % num2),
            target.reg_label().unwrap(),
        )?;
        Self::inc_ptr(exa);
        Ok(())
    }

    fn swiz(
        &self,
        exa: &Rc<RefCell<Exa>>,
        num1: Arg,
        num2: Arg,
        target: Arg,
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
        Self::inc_ptr(exa);
        Ok(())
    }

    fn rand(
        &self,
        exa: &Rc<RefCell<Exa>>,
        num1: Arg,
        num2: Arg,
        target: Arg,
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
        Self::inc_ptr(exa);
        Ok(())
    }

    fn test(
        &self,
        exa: &Rc<RefCell<Exa>>,
        arg1: Arg,
        comp: Arg,
        arg2: Arg,
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
        Self::inc_ptr(exa);
        Ok(())
    }

    fn tjmp(&self, exa: &Rc<RefCell<Exa>>, target: Arg) -> Result<(), ExaResult> {
        if exa.borrow().reg_t == Register::Number(0) {
            Self::inc_ptr(exa);
            return Ok(());
        }
        self.jump(exa, target)
    }

    fn fjmp(&self, exa: &Rc<RefCell<Exa>>, target: Arg) -> Result<(), ExaResult> {
        if exa.borrow().reg_t != Register::Number(0) {
            Self::inc_ptr(exa);
            return Ok(());
        }
        self.jump(exa, target)
    }

    fn jump(&self, exa: &Rc<RefCell<Exa>>, target: Arg) -> Result<(), ExaResult> {
        let mut x = 0;
        let instr_list = { exa.borrow().instr_list.clone() };
        for instr in instr_list.iter() {
            if instr.0 != Instruction::Mark {
                x += 1;
                continue;
            }
            if instr.1.as_ref().unwrap() == &target {
                exa.borrow_mut().instr_ptr = x;
                Self::inc_ptr(exa);
                return Ok(());
            }
        }
        Err(ExaResult::Error(RuntimeError::LabelNotFound))
    }

    fn repl(&self, exa: &Rc<RefCell<Exa>>, target: Arg) -> Result<(), ExaResult> {
        let ptr = { exa.borrow().instr_ptr };
        self.jump(exa, target)?;
        let traget_ptr = { exa.borrow().instr_ptr };
        exa.borrow_mut().instr_ptr = ptr;
        Self::inc_ptr(exa);
        Err(ExaResult::SideEffect(SideEffect::Repl(traget_ptr)))
    }

    fn host(&self, exa: &Rc<RefCell<Exa>>, target: Arg) -> Result<(), ExaResult> {
        // TODO: make this actually use hostname
        self.put_value(
            exa,
            Register::Keyword("Rhizome".to_string()),
            target.reg_label().unwrap(),
        )?;
        Self::inc_ptr(exa);
        Ok(())
    }

    fn prnt(&self, exa: &Rc<RefCell<Exa>>, target: Arg) -> Result<(), ExaResult> {
        println!("{}> {}", exa.borrow().name, self.get_value(exa, target)?);
        Self::inc_ptr(exa);
        Ok(())
    }

    fn get_number(&self, exa: &Rc<RefCell<Exa>>, target: Arg) -> Result<i16, ExaResult> {
        match self.get_value(exa, target)? {
            Register::Number(n) => Ok(n),
            Register::Keyword(_) => Err(ExaResult::Error(RuntimeError::MathWithKeywords)),
        }
    }

    fn get_value(&self, exa: &Rc<RefCell<Exa>>, target: Arg) -> Result<Register, ExaResult> {
        match target {
            Arg::Number(n) => Ok(Register::Number(n)),
            Arg::Keyword(k) => Ok(Register::Keyword(k)),
            Arg::RegLabel(r) => match r {
                RegLabel::X => Ok(exa.borrow().reg_x.clone()),
                RegLabel::T => Ok(exa.borrow().reg_t.clone()),
                RegLabel::F => Err(ExaResult::Error(RuntimeError::InvalidFileAccess)),
                RegLabel::M => match self.reg_m.borrow_mut().take() {
                    Some(r) => Ok(r),
                    None => Err(ExaResult::SideEffect(SideEffect::Recv)),
                },
                RegLabel::H(_) => Err(ExaResult::Error(RuntimeError::InvalidHWRegAccess)),
            },
            _ => Err(ExaResult::Error(RuntimeError::InvalidArgument)),
        }
    }

    fn put_value(
        &self,
        exa: &Rc<RefCell<Exa>>,
        value: Register,
        target: RegLabel,
    ) -> Result<(), ExaResult> {
        match target {
            RegLabel::X => {
                exa.borrow_mut().reg_x = value;
                Ok(())
            }
            RegLabel::T => {
                exa.borrow_mut().reg_t = value;
                Ok(())
            }
            RegLabel::F => Err(ExaResult::Error(RuntimeError::InvalidFileAccess)),
            RegLabel::M => {
                let mut reg_m = self.reg_m.borrow_mut();
                if reg_m.is_some() {
                    Err(ExaResult::SideEffect(SideEffect::Send))
                } else {
                    *reg_m = Some(value);
                    Ok(())
                }
            }
            RegLabel::H(_) => Err(ExaResult::Error(RuntimeError::InvalidHWRegAccess)),
        }
    }

    fn inc_ptr(exa: &Rc<RefCell<Exa>>) {
        exa.borrow_mut().instr_ptr += 1;
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}
