use std::collections::HashMap;

use crate::config::VMConfig as Config;
use crate::exa::{Block, ExaStatus, SideEffect};
use crate::exa::{Exa, PackedExa};
use crate::runtime::Runtime;

pub struct VM {
    exas: HashMap<usize, Exa>,
    runtime: Runtime,
}

impl VM {
    pub fn new(hostname: &str, config: Config) -> Self {
        Self {
            exas: HashMap::with_capacity(config.max_exas),
            runtime: Runtime::new(hostname),
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
            exa.hydrate(self.runtime.get_harness()),
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

impl Default for VM {
    fn default() -> Self {
        Self::new("Rhizome".into(), Config::default())
    }
}
