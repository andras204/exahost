use std::collections::HashMap;

use crate::exa::{Exa, ExaResult, VMRequest};

#[derive(Debug)]
pub struct ExaVM {
    ready: HashMap<String, Exa>,
    send: HashMap<String, Exa>,
    recv: HashMap<String, Exa>,
    link_reqs: Vec<(i16, Exa)>,
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
        let results: HashMap<String, Result<(), ExaResult>> = self
            .ready
            .iter_mut()
            .map(|(k, e)| (k.clone(), e.exec()))
            .collect();
        self.process_results(results);
        self.handle_m_register();
    }

    fn process_results(&mut self, results: HashMap<String, Result<(), ExaResult>>) {
        for (k, res) in results.iter() {
            match res {
                Ok(_) => (),
                Err(r) => match r {
                    ExaResult::Error(e) => {
                        println!("[VM] Error with {}: {:?}", k, e);
                        self.halt_exa(k);
                    }
                    ExaResult::VMRequest(rq) => match rq {
                        VMRequest::Halt => self.halt_exa(k),
                        VMRequest::Kill => self.kill_exa(k),
                        VMRequest::Repl(c) => self.add_clone(c),
                        VMRequest::Tx => {
                            let (n, e) = self.ready.remove_entry(k).unwrap();
                            self.send.insert(n, e);
                        }
                        VMRequest::Rx => {
                            let (n, e) = self.ready.remove_entry(k).unwrap();
                            self.recv.insert(n, e);
                        }
                        VMRequest::Link(l) => {
                            let exa = self.ready.remove(k).unwrap();
                            self.link_reqs.push((l.to_owned(), exa));
                        }
                    },
                },
            }
        }
    }

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
