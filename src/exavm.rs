use std::collections::HashMap;

use crate::{
    exa::Exa, 
    signal::{ExaSignal, HostSignal},
};

pub struct ExaVM {
    ready: HashMap<String, Exa>,
    send: HashMap<String, Exa>,
    recv: HashMap<String, Exa>,
    link_stack: Vec<HostSignal>
}

impl ExaVM {
    pub fn new() -> Self {
        ExaVM {
            ready: HashMap::new(),
            send: HashMap::new(),
            recv: HashMap::new(),
            link_stack: Vec::new(),
        }
    }

    pub fn add_exa(&mut self, exa: Exa) {
        self.ready.insert(exa.name.clone(), exa);
    }

    pub fn add_clone(&mut self, exa: &Exa) {
        self.ready.insert(exa.name.clone(), exa.to_owned());
    }

    pub fn step(&mut self) -> Option<HostSignal> {
        let results: HashMap<String, ExaSignal> = self.ready.iter_mut()
            .map(|(k, e)| (k.clone(), e.exec()))
            .collect();
        self.process_results(results);
        self.handle_m_register();
        self.link_stack.pop()
    }

    fn process_results(&mut self, results: HashMap<String, ExaSignal>) {
        for (k, r) in results.iter() {
            match r {
                ExaSignal::Ok => (),
                ExaSignal::Err(e) => {
                    eprintln!("[VM]: Error with {}: {}", k, e);
                    self.halt_exa(k);
                },
                ExaSignal::Halt => self.halt_exa(k),
                ExaSignal::Kill => self.kill_exa(k),
                ExaSignal::Repl(e) => self.add_clone(e),
                ExaSignal::Tx => {
                    let (n, e) = self.ready.remove_entry(k).unwrap();
                    self.send.insert(n, e);
                },
                ExaSignal::Rx => {
                    let (n, e) = self.ready.remove_entry(k).unwrap();
                    self.recv.insert(n, e);
                },
                ExaSignal::Link(l) => {
                    let exa = self.ready.remove(k).unwrap();
                    self.link_stack.push(HostSignal::Link((l.to_owned(), exa)));
                }
            }
        }
    }

    fn handle_m_register(&mut self) {
        if self.send.len() < 1 || self.recv.len() < 1 { return; }
        let mut k = self.send.keys().nth(0).unwrap().clone();
        let mut send = self.send.remove(&k).unwrap();
        k = self.recv.keys().nth(0).unwrap().clone();
        let mut recv = self.recv.remove(&k).unwrap();

        recv.m_reg = send.send_m();

        self.ready.insert(send.name.clone(), send);

        match recv.exec() {
            ExaSignal::Err(e) => {
                eprintln!("[VM]: Error with {}: {}", recv.name.clone(), e);
                return;
            },
            ExaSignal::Tx => {
                self.send.insert(recv.name.clone(), recv);
                return;
            },
            ExaSignal::Rx => {
                self.recv.insert(recv.name.clone(), recv);
                return;
            },
            ExaSignal::Link(l) => {
                self.link_stack.push(HostSignal::Link((l.to_owned(), recv)));
                return;
            },
            _ => (), 
        }
        self.ready.insert(recv.name.clone(), recv);
    }

    fn halt_exa(&mut self, name: &String) {
        self.ready.remove(name);
    }

    fn kill_exa(&mut self, name: &String) {
        for k in self.ready.keys() {
            if k != name {
                self.ready.remove(name);
                return;
            }
        }
        if self.send.len() > 0 {
            let k = self.send.keys().nth(0).unwrap().clone();
            self.send.remove(&k);
            return;
        }
        if self.recv.len() > 0 {
            let k = self.recv.keys().nth(0).unwrap().clone();
            self.recv.remove(&k);
            return;
        }
    }
}
