use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::exa::{Arg, Exa, Instruction, RegLabel, Register};

#[derive(Debug)]
pub struct ExaVM {
    ready: HashMap<String, Exa>,
    send: HashMap<String, Exa>,
    recv: HashMap<String, Exa>,
    link_reqs: Vec<(i16, Exa)>,
}

enum ExaResult {
    SideEffect(SideEffect),
    Error(RuntimeError),
}

enum SideEffect {
    Halt(String),
    Kill(String),
    Send(String),
    Recv(String),
    Error(String, RuntimeError),
}

enum RuntimeError {
    OutOfInstructions,
    UnsupportedInstruction,
    InvalidFileAccess,
    InvalidHWRegAccess,
    InvalidArgument,
    MathWithKeywords,
}

#[derive(Debug)]
pub struct VM {
    exas: HashMap<String, Rc<RefCell<Exa>>>,
    reg_m: RefCell<Option<Register>>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            exas: HashMap::new(),
            reg_m: RefCell::new(None),
        }
    }

    pub fn add_exa(&mut self, exa: Exa) {
        self.exas
            .insert(exa.clone().name, Rc::new(RefCell::new(exa)));
    }

    pub fn exec_all(&mut self) {
        let mut side_effects = Vec::with_capacity(self.exas.len());
        for exa in self.exas.values() {
            if let Err(res) = self.exec(exa.clone()) {
                match res {
                    ExaResult::SideEffect(se) => {
                        side_effects.push(se);
                    }
                    ExaResult::Error(e) => {
                        side_effects.push(SideEffect::Error("XA".to_string(), e));
                    }
                }
            }
        }
    }

    fn exec(&self, exa: Rc<RefCell<Exa>>) -> Result<(), ExaResult> {
        let (instr, args) = {
            let e = exa.borrow();
            if e.instr_ptr as usize == e.instr_list.len() {
                return Err(ExaResult::Error(RuntimeError::OutOfInstructions));
            }
            e.instr_list[e.instr_ptr as usize].clone()
        };
        let args = args.unwrap_or(vec![]);
        let result = match instr {
            Instruction::Copy => self.copy(exa.clone(), args[0].clone(), args[1].clone()),
            Instruction::Addi => self.addi(
                exa.clone(),
                args[0].clone(),
                args[1].clone(),
                args[2].clone(),
            ),
            Instruction::Halt => Err(ExaResult::SideEffect(SideEffect::Halt("XA".to_string()))),
            _ => Err(ExaResult::Error(RuntimeError::UnsupportedInstruction)),
        };
        if result.is_ok() {
            exa.borrow_mut().instr_ptr += 1;
        }
        result
    }

    fn copy(&self, exa: Rc<RefCell<Exa>>, value: Arg, target: Arg) -> Result<(), ExaResult> {
        let val = self.get_value(&exa, value)?;
        self.put_value(&exa, val, target.reg_label().unwrap())?;
        Ok(())
    }

    fn addi(
        &self,
        exa: Rc<RefCell<Exa>>,
        num1: Arg,
        num2: Arg,
        target: Arg,
    ) -> Result<(), ExaResult> {
        let num1 = self.get_number(&exa, num1)?;
        let num2 = self.get_number(&exa, num2)?;
        self.put_value(
            &exa,
            Register::Number(num1 + num2),
            target.reg_label().unwrap(),
        )
    }

    fn get_number(&self, exa: &Rc<RefCell<Exa>>, target: Arg) -> Result<i16, ExaResult> {
        fn reg_to_i16(reg: Register) -> Result<i16, ExaResult> {
            match reg {
                Register::Number(n) => Ok(n),
                Register::Keyword(k) => Err(ExaResult::Error(RuntimeError::MathWithKeywords)),
            }
        }
        reg_to_i16(self.get_value(exa, target)?)
    }

    fn get_value(&self, exa: &Rc<RefCell<Exa>>, target: Arg) -> Result<Register, ExaResult> {
        match target {
            Arg::Number(n) => Ok(Register::Number(n)),
            Arg::Keyword(k) => Ok(Register::Keyword(k)),
            Arg::RegLabel(r) => match r {
                RegLabel::X => Ok(exa.borrow().reg_x.clone()),
                RegLabel::T => Ok(exa.borrow().reg_t.clone()),
                RegLabel::F => Err(ExaResult::Error(RuntimeError::InvalidFileAccess)),
                RegLabel::M => Ok(self.reg_m.borrow_mut().take().unwrap()),
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
                *self.reg_m.borrow_mut() = Some(value);
                Ok(())
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

impl Default for ExaVM {
    fn default() -> Self {
        ExaVM::new()
    }
}

impl ExaVM {
    pub fn new() -> Self {
        ExaVM {
            ready: HashMap::new(),
            send: HashMap::new(),
            recv: HashMap::new(),
            link_reqs: Vec::new(),
        }
    }

    pub fn add_exa(&mut self, exa: Exa) {
        self.ready.insert(exa.name.clone(), exa);
    }

    fn add_clone(&mut self, exa: &Exa) {
        self.ready.insert(exa.name.clone(), exa.to_owned());
    }

    pub fn step(&mut self) {
        // let results: HashMap<String, Result<(), ExaResult>> = self
        //     .ready
        //     .iter_mut()
        //     .map(|(k, e)| (k.clone(), e.exec()))
        //     .collect();
        // self.process_results(results);
        // self.handle_m_register();
    }

    // fn process_results(&mut self, results: HashMap<String, Result<(), ExaResult>>) {
    //     for (k, res) in results.iter() {
    //         match res {
    //             Ok(_) => (),
    //             Err(r) => match r {
    //                 ExaResult::Error(e) => {
    //                     println!("[VM] Error with {}: {:?}", k, e);
    //                     self.halt_exa(k);
    //                 }
    //                 ExaResult::VMRequest(rq) => match rq {
    //                     VMRequest::Halt => self.halt_exa(k),
    //                     VMRequest::Kill => self.kill_exa(k),
    //                     VMRequest::Repl(c) => self.add_clone(c),
    //                     VMRequest::Tx => {
    //                         let (n, e) = self.ready.remove_entry(k).unwrap();
    //                         self.send.insert(n, e);
    //                     }
    //                     VMRequest::Rx => {
    //                         let (n, e) = self.ready.remove_entry(k).unwrap();
    //                         self.recv.insert(n, e);
    //                     }
    //                     VMRequest::Link(l) => {
    //                         let exa = self.ready.remove(k).unwrap();
    //                         self.link_reqs.push((l.to_owned(), exa));
    //                     }
    //                 },
    //             },
    //         }
    //     }
    // }
    //
    fn handle_m_register(&mut self) {
        if self.send.is_empty() || self.recv.is_empty() {
            return;
        }
        let mut k = self.send.keys().nth(0).unwrap().clone();
        let mut send = self.send.remove(&k).unwrap();
        k = self.recv.keys().nth(0).unwrap().clone();
        let mut recv = self.recv.remove(&k).unwrap();

        recv.reg_m = send.send_m();

        self.ready.insert(send.name.clone(), send);

        // call .exec on recv and handle it here to
        // get 1 cycle instructions even with M access

        self.ready.insert(recv.name.clone(), recv);
    }

    fn halt_exa(&mut self, name: &String) {
        self.ready.remove(name).unwrap();
    }

    fn kill_exa(&mut self, name: &String) {
        for k in self.ready.keys() {
            if k != name {
                self.ready.remove(name);
                return;
            }
        }
        if !self.send.is_empty() {
            let k = self.send.keys().nth(0).unwrap().clone();
            self.send.remove(&k);
            return;
        }
        if !self.recv.is_empty() {
            let k = self.recv.keys().nth(0).unwrap().clone();
            self.recv.remove(&k);
        }
    }
}
