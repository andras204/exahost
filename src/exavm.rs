use std::collections::HashMap;

use crate::exa::{Exa, ExaResult, VMRequest};

#[derive(Debug)]
pub struct ExaVM {
    ready: HashMap<String, Exa>,
    send: HashMap<String, Exa>,
    recv: HashMap<String, Exa>,
    link_reqs: Vec<(i16, Exa)>,
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
}

impl ExaVM {
    fn exec(&mut self, exa: Exa) {
        match exa.instr_list[exa.instr_ptr] {}
    }
}
