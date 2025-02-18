use std::collections::HashMap;
use std::sync::Arc;

use bridge::VMBridge;
use log::info;
use rand::{rngs::ThreadRng, Rng};
use tokio::sync::Mutex;

use crate::exa::{Block, ExaStatus, SideEffect};
use crate::exa::{Exa, PackedExa};
use crate::runtime::SharedRT;

pub mod bridge;

#[derive(Debug)]
pub struct VM {
    exas: HashMap<usize, Exa>,
    rng: ThreadRng,
    rt_ref: SharedRT,
    bridge: Arc<Mutex<VMBridge>>,
}

impl VM {
    pub fn new(rt_ref: SharedRT, max_capacity: usize) -> Self {
        Self {
            exas: HashMap::new(),
            rng: rand::rng(),
            rt_ref,
            bridge: Arc::new(Mutex::new(VMBridge::new(max_capacity))),
        }
    }

    pub fn get_bridge(&self) -> Arc<Mutex<VMBridge>> {
        self.bridge.clone()
    }

    pub fn step(&mut self) {
        if self.exas.is_empty() {
            return;
        }
        let results = self.exec_all();
        self.apply_side_effects(results);
    }

    pub fn add_exa(&mut self, exa: PackedExa) -> Result<(), PackedExa> {
        if self.is_full() {
            return Err(exa);
        }
        self.add_exa_internal(exa.hydrate(self.rt_ref.clone()))
            .unwrap();
        Ok(())
    }

    fn exec_all(&mut self) -> Vec<(usize, ExaStatus)> {
        let results: Vec<(usize, ExaStatus)> = self
            .exas
            .iter_mut()
            .filter_map(|(k, exa)| match exa.exec() {
                Ok(_) => None,
                Err(r) => Some((*k, r)),
            })
            .collect();
        results
    }

    fn apply_side_effects(&mut self, results: Vec<(usize, ExaStatus)>) {
        for (k, res) in results {
            match res {
                ExaStatus::Block(b) => match b {
                    Block::Recv => {
                        let _ = self.exas.get_mut(&k).unwrap().exec();
                    }
                    Block::Repl(j) => {
                        self.generate_clone(&k, j);
                    }
                    _ => (),
                },
                ExaStatus::SideEffect(se) => match se {
                    SideEffect::Kill => self.kill(k),
                    SideEffect::Link(l) => {
                        let exa = self.exas.remove(&k).unwrap();
                        self.bridge
                            .blocking_lock()
                            .txfer_to_outgoing(k, l, exa.pack());
                    }
                },
                ExaStatus::Error(e) => {
                    let name = self.remove_exa_internal(&k).name;
                    info!("Exa error: {} |> {:?}", name, e);
                }
            }
        }
    }

    fn kill(&mut self, k: usize) {
        let mut bridge = self.bridge.blocking_lock();
        let len = self.exas.len() + bridge.outgoing_len();
        let r = self.rng.random_ratio(self.exas.len() as u32, len as u32);
        if r {
            for k2 in self.exas.keys() {
                if k2 != &k {
                    self.exas.remove(&k).unwrap();
                    break;
                }
            }
        } else {
            for k2 in bridge.keys() {
                if k2 != &k {
                    bridge.remove_outgoing(&k);
                    break;
                }
            }
        }
        bridge.update_capacity(self.exas.len());
    }

    fn generate_clone(&mut self, k: &usize, j: u8) {
        if self.is_full() {
            return;
        }

        let original = self.exas.get_mut(k).unwrap();

        original.repl_counter += 1;
        original.instr_ptr += 1;

        let mut clone = original.clone();

        clone.instr_ptr = j;
        clone.repl_counter = 0;
        clone.name.push_str(&format!(":{}", original.repl_counter));

        self.add_exa_internal(clone).unwrap();
    }

    fn add_exa_internal(&mut self, exa: Exa) -> Result<(), ()> {
        if self.is_full() {
            return Err(());
        }
        self.exas
            .insert(self.exas.keys().max().unwrap_or(&0) + 1, exa);
        self.update_cap();
        Ok(())
    }

    fn remove_exa_internal(&mut self, k: &usize) -> Exa {
        let exa = self.exas.remove(k).unwrap();
        self.update_cap();
        exa
    }

    fn is_full(&self) -> bool {
        self.bridge.blocking_lock().is_full()
    }

    fn update_cap(&self) {
        self.bridge.blocking_lock().update_capacity(self.exas.len());
    }
}
