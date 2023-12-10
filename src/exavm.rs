use std::sync::mpsc::Sender;

use self::exa::Exa;

pub mod exa;

pub struct ExaVM {
    return_tx: Sender<String>,
    exas: Vec<Exa>,
}

impl ExaVM {
    pub fn new(return_tx: Sender<String>) -> ExaVM {
        ExaVM {
            return_tx,
            exas: Vec::new(),
        }
    }

    pub fn add_exa(&mut self, exa: Exa) {
        self.exas.push(exa);
    }

    pub fn step(&mut self) {
        let results: Vec<_> = self.exas.iter_mut().map(|e| e.exec()).collect();
    }
}
