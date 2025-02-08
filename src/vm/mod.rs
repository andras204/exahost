use std::collections::HashMap;

use crate::exa::{Block, ExaStatus, SideEffect};
use crate::exa::{Exa, PackedExa};
use crate::runtime::WeakRT;

pub struct VM {
    exas: HashMap<usize, Exa>,
    rt_ref: WeakRT,
}

impl VM {
    pub fn new(rt_ref: WeakRT) -> Self {
        Self {
            exas: HashMap::new(),
            rt_ref,
        }
    }

    pub fn step(&mut self) {
        if self.exas.is_empty() {
            return;
        }
        let results = self.exec_all();
        self.apply_side_effects(results);
    }

    pub fn add_exa(&mut self, exa: PackedExa) {
        self.exas.insert(
            match self.exas.keys().max() {
                Some(n) => n + 1,
                None => 0,
            },
            exa.hydrate(self.rt_ref.clone()),
        );
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
                    _ => (),
                },
                ExaStatus::SideEffect(se) => match se {
                    SideEffect::Repl(j) => {
                        self.generate_clone(&k, j);
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
                ExaStatus::Error(e) => {
                    let name = self.exas.remove(&k).unwrap().name;
                    println!("{}| {:?}", name, e);
                }
            }
        }
    }

    fn generate_clone(&mut self, k: &usize, j: u8) {
        let original = self.exas.get_mut(k).unwrap();
        original.repl_counter += 1;

        let mut clone = original.clone();

        clone.instr_ptr = j;
        clone.name.push_str(&format!(":{}", clone.repl_counter));
        clone.repl_counter = 0;

        self.exas.insert(self.exas.keys().max().unwrap() + 1, clone);
    }
}
