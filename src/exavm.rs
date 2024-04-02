use std::{cell::RefCell, collections::HashMap, rc::Rc};

use rand::{rngs::ThreadRng, Rng};

use crate::exa::{Arg, Comp, Exa, Instruction, RegLabel, Register};
use crate::file::File;

#[derive(Debug, Clone, Copy)]
enum ExaResult {
    SideEffect(SideEffect),
    Interrupt(Interrupt),
    Error(RuntimeError),
}

impl ExaResult {
    pub fn is_interrupt(&self) -> bool {
        matches!(self, Self::Interrupt(_))
    }
}

#[derive(Debug, Clone, Copy)]
enum SideEffect {
    Repl(u8),
    Link(i16),
    Kill,
}

#[derive(Debug, Clone, Copy)]
enum Interrupt {
    Send,
    Recv,
    Halt,
}

#[derive(Debug, Clone, Copy)]
enum RuntimeError {
    OutOfInstructions,
    UnsupportedInstruction,
    InvalidFileAccess,
    InvalidHWRegAccess,
    InvalidFRegAccess,
    InvalidArgument,
    MathWithKeywords,
    LabelNotFound,
    AlreadyHoldingFile,
}

#[derive(Debug)]
pub struct VM {
    exas: HashMap<usize, Rc<RefCell<Exa>>>,
    // send: HashSet<usize>,
    // recv: HashSet<usize>,
    reg_m: RefCell<Option<Register>>,
    rng: RefCell<ThreadRng>,

    files: RefCell<HashMap<i16, File>>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            exas: HashMap::new(),
            // send: HashSet::new(),
            // recv: HashSet::new(),
            reg_m: RefCell::new(None),
            rng: RefCell::new(rand::thread_rng()),
            files: RefCell::new(HashMap::new()),
        }
    }

    pub fn step(&mut self) {
        let results = self.exec_all();
        self.apply_side_effects(results);
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
        for (i, exa) in self.exas.iter()
        // .filter(|(k, _)| !(self.recv.contains(k) || self.send.contains(k)))
        {
            if let Err(res) = self.exec(exa.clone()) {
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
                        let (key, val) = self.add_clone(&k, j);
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
                },
                ExaResult::Interrupt(it) => match it {
                    Interrupt::Recv => {
                        // self.recv.insert(k);
                    }
                    Interrupt::Send => {
                        // self.send.insert(k);
                    }
                    Interrupt::Halt => {
                        self.exas.remove(&k);
                    }
                },
                ExaResult::Error(e) => {
                    let name = self.exas.remove(&k).unwrap().borrow().name.clone();
                    println!("{}| {:?}", name, e);
                }
            }
        }
    }

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

        let res = match instr {
            Instruction::Copy => self.copy(&exa, arg1.unwrap(), arg2.unwrap()),
            Instruction::Void => self.void(&exa, arg1.unwrap()),

            Instruction::Addi => self.addi(&exa, arg1.unwrap(), arg2.unwrap(), arg3.unwrap()),
            Instruction::Subi => self.subi(&exa, arg1.unwrap(), arg2.unwrap(), arg3.unwrap()),
            Instruction::Muli => self.muli(&exa, arg1.unwrap(), arg2.unwrap(), arg3.unwrap()),
            Instruction::Divi => self.divi(&exa, arg1.unwrap(), arg2.unwrap(), arg3.unwrap()),
            Instruction::Modi => self.modi(&exa, arg1.unwrap(), arg2.unwrap(), arg3.unwrap()),
            Instruction::Swiz => self.swiz(&exa, arg1.unwrap(), arg2.unwrap(), arg3.unwrap()),
            Instruction::Rand => self.rand(&exa, arg1.unwrap(), arg2.unwrap(), arg3.unwrap()),

            Instruction::Test => self.test(&exa, arg1.unwrap(), arg2.unwrap(), arg3.unwrap()),
            Instruction::TestMrd => self.test_mrd(&exa),
            Instruction::TestEof => Self::test_eof(&exa),

            Instruction::Jump => Self::jump(&exa, arg1.unwrap()),
            Instruction::Tjmp => Self::tjmp(&exa, arg1.unwrap()),
            Instruction::Fjmp => Self::fjmp(&exa, arg1.unwrap()),

            Instruction::Make => self.make(&exa),
            Instruction::Grab => self.grab(&exa, arg1.unwrap()),
            Instruction::File => self.file(&exa, arg1.unwrap()),
            Instruction::Seek => self.seek(&exa, arg1.unwrap()),
            Instruction::Drop => self.drop(&exa),
            Instruction::Wipe => Self::wipe(&exa),

            Instruction::Link => Err(ExaResult::SideEffect(SideEffect::Link(
                arg1.unwrap().number().unwrap(),
            ))),
            Instruction::Repl => Self::repl(&exa, arg1.unwrap()),
            Instruction::Halt => Err(ExaResult::Interrupt(Interrupt::Halt)),
            Instruction::Kill => Err(ExaResult::SideEffect(SideEffect::Kill)),

            Instruction::Host => self.host(&exa, arg1.unwrap()),
            Instruction::Noop => Ok(()),

            Instruction::Mark => Err(ExaResult::Error(RuntimeError::UnsupportedInstruction)),

            Instruction::Prnt => self.prnt(&exa, arg1.unwrap()),
        };

        if let Err(e) = res {
            if e.is_interrupt() {
                return Err(e);
            }
        }
        exa.borrow_mut().instr_ptr += 1;
        res
    }

    fn copy(&self, exa: &Rc<RefCell<Exa>>, value: Arg, target: Arg) -> Result<(), ExaResult> {
        let val = self.get_value(exa, value)?;
        self.put_value(exa, val, target.reg_label().unwrap())?;
        Ok(())
    }

    fn void(&self, exa: &Rc<RefCell<Exa>>, target: Arg) -> Result<(), ExaResult> {
        match target.reg_label().unwrap() {
            RegLabel::X => exa.borrow_mut().reg_x = Register::Number(0),
            RegLabel::T => exa.borrow_mut().reg_t = Register::Number(0),
            RegLabel::F => self.put_value(exa, Register::Keyword("".to_string()), RegLabel::F)?,
            RegLabel::M => match self.reg_m.borrow_mut().take() {
                Some(_) => return Ok(()),
                None => return Err(ExaResult::Interrupt(Interrupt::Recv)),
            },
            RegLabel::H(_) => return Err(ExaResult::Error(RuntimeError::InvalidHWRegAccess)),
        }
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
        Ok(())
    }

    fn test_eof(exa: &Rc<RefCell<Exa>>) -> Result<(), ExaResult> {
        let eof = match exa.borrow().reg_f.as_ref() {
            Some(f) => f.1.is_eof(),
            None => return Err(ExaResult::Error(RuntimeError::InvalidFRegAccess)),
        } as i16;
        {
            exa.borrow_mut().reg_t = Register::Number(eof);
        }
        Ok(())
    }

    fn test_mrd(&self, exa: &Rc<RefCell<Exa>>) -> Result<(), ExaResult> {
        {
            exa.borrow_mut().reg_t = Register::Number(self.reg_m.borrow().is_some() as i16);
        }
        Ok(())
    }

    fn tjmp(exa: &Rc<RefCell<Exa>>, target: Arg) -> Result<(), ExaResult> {
        if exa.borrow().reg_t == Register::Number(0) {
            return Ok(());
        }
        Self::jump(exa, target)
    }

    fn fjmp(exa: &Rc<RefCell<Exa>>, target: Arg) -> Result<(), ExaResult> {
        if exa.borrow().reg_t != Register::Number(0) {
            return Ok(());
        }
        Self::jump(exa, target)
    }

    fn jump(exa: &Rc<RefCell<Exa>>, target: Arg) -> Result<(), ExaResult> {
        let res = exa
            .borrow()
            .instr_list
            .iter()
            .position(|i| i.0 == Instruction::Mark && i.1.as_ref().unwrap() == &target);
        match res {
            Some(x) => {
                exa.borrow_mut().instr_ptr = x as u8;
                Ok(())
            }
            None => Err(ExaResult::Error(RuntimeError::LabelNotFound)),
        }
    }

    fn make(&self, exa: &Rc<RefCell<Exa>>) -> Result<(), ExaResult> {
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
            File::new(),
        ));
        Ok(())
    }

    fn grab(&self, exa: &Rc<RefCell<Exa>>, target: Arg) -> Result<(), ExaResult> {
        match self
            .files
            .borrow_mut()
            .remove_entry(&target.number().unwrap())
        {
            Some(t) => {
                exa.borrow_mut().reg_f = Some(t);
                Ok(())
            }
            None => Err(ExaResult::Error(RuntimeError::InvalidFileAccess)),
        }
    }

    fn file(&self, exa: &Rc<RefCell<Exa>>, arg1: Arg) -> Result<(), ExaResult> {
        let f = match exa.borrow().reg_f.as_ref() {
            Some(f) => Ok(f.0),
            None => Err(ExaResult::Error(RuntimeError::InvalidFRegAccess)),
        }?;
        self.put_value(exa, Register::Number(f), arg1.reg_label().unwrap())
    }

    fn seek(&self, exa: &Rc<RefCell<Exa>>, arg1: Arg) -> Result<(), ExaResult> {
        let n = self.get_number(exa, arg1)?;
        match exa.borrow_mut().reg_f.as_mut() {
            Some(f) => {
                f.1.seek(n);
                Ok(())
            }
            None => Err(ExaResult::Error(RuntimeError::InvalidFRegAccess)),
        }
    }

    fn drop(&self, exa: &Rc<RefCell<Exa>>) -> Result<(), ExaResult> {
        if let Some(f) = exa.borrow_mut().reg_f.take() {
            self.files.borrow_mut().insert(f.0, f.1);
            return Ok(());
        }
        Err(ExaResult::Error(RuntimeError::InvalidFileAccess))
    }

    fn wipe(exa: &Rc<RefCell<Exa>>) -> Result<(), ExaResult> {
        if exa.borrow_mut().reg_f.take().is_some() {
            return Ok(());
        }
        Err(ExaResult::Error(RuntimeError::InvalidFileAccess))
    }

    fn repl(exa: &Rc<RefCell<Exa>>, target: Arg) -> Result<(), ExaResult> {
        let ptr = { exa.borrow().instr_ptr };
        Self::jump(exa, target)?;
        let traget_ptr = { exa.borrow().instr_ptr };
        exa.borrow_mut().instr_ptr = ptr;
        Err(ExaResult::SideEffect(SideEffect::Repl(traget_ptr)))
    }

    fn host(&self, exa: &Rc<RefCell<Exa>>, target: Arg) -> Result<(), ExaResult> {
        // TODO: make this actually use hostname
        self.put_value(
            exa,
            Register::Keyword("Rhizome".to_string()),
            target.reg_label().unwrap(),
        )?;
        Ok(())
    }

    fn prnt(&self, exa: &Rc<RefCell<Exa>>, target: Arg) -> Result<(), ExaResult> {
        let name = { exa.borrow().name.clone() };
        println!("{}> {}", name, self.get_value(exa, target)?);
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
                RegLabel::F => {
                    if let Some(f_ref) = exa.borrow_mut().reg_f.as_mut() {
                        match f_ref.1.read() {
                            Some(r) => Ok(r),
                            None => Err(ExaResult::Error(RuntimeError::InvalidFRegAccess)),
                        }
                    } else {
                        Err(ExaResult::Error(RuntimeError::InvalidFRegAccess))
                    }
                }
                RegLabel::M => match self.reg_m.borrow_mut().take() {
                    Some(r) => Ok(r),
                    None => Err(ExaResult::Interrupt(Interrupt::Recv)),
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
                    Err(ExaResult::Interrupt(Interrupt::Send))
                } else {
                    *reg_m = Some(value);
                    Ok(())
                }
            }
            RegLabel::H(_) => Err(ExaResult::Error(RuntimeError::InvalidHWRegAccess)),
        }
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}
