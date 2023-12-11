use std::sync::{mpsc::Sender, Arc};

use crate::{
    exa::{Exa, Register}, 
    signal::{ExaSignal, HostSignal},
};

pub struct ExaVM {
    return_tx: Sender<HostSignal>,
    exas: Vec<Exa>,
    reg_m: Arc<Option<Register>>,
}

impl ExaVM {
    pub fn new(return_tx: Sender<HostSignal>) -> ExaVM {
        ExaVM {
            return_tx,
            exas: Vec::new(),
            reg_m: Arc::new(Some(Register::Number(0))),
        }
    }

    pub fn add_exa(&mut self, exa: Exa) {
        self.exas.push(exa);
    }

    pub fn step(&mut self) {
        let results: Vec<_> = self.exas.iter_mut()
            .map(|e| e.exec())
            .collect();
        self.process_results(results);
    }

    fn process_results(&mut self, results: Vec<ExaSignal>) {
        for x in 0..results.len() {
            match results[x].clone() {
                ExaSignal::Ok => (),
                ExaSignal::Err(_) => {
                    self.halt_exa(x);
                },
                ExaSignal::Halt => self.halt_exa(x),
                ExaSignal::Kill => self.kill_exa(x),
                ExaSignal::Repl(e) => self.add_exa(e),
                _ => (),
            }
        }
    }

    fn halt_exa(&mut self, index: usize) {
        self.exas.remove(index);
    }

    fn kill_exa(&mut self, index: usize) {
        for x in 0..self.exas.len() {
            if x != index { self.exas.remove(x); }
        }
    }
}
