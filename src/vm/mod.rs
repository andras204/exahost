use log::*;
use nohash_hasher::IntMap;
use rand::{rngs::ThreadRng, seq::IteratorRandom, Rng};

use crate::{
    backbone::{capacity_pool::CapacityToken, Backbone},
    exa::{status::*, Exa, PackedExa},
};

pub mod runtime;

use runtime::Runtime;

#[derive(Debug)]
pub struct VM {
    exas: IntMap<usize, Exa>,
    rng: ThreadRng,
    rt: Runtime,
    backbone: Backbone,
    cap_tokens: Vec<CapacityToken>,
}

impl VM {
    pub fn new(rt: Runtime, backbone: Backbone) -> Self {
        Self {
            exas: IntMap::default(),
            rng: rand::rng(),
            rt,
            backbone,
            cap_tokens: Vec::new(),
        }
    }

    pub fn add_exa(&mut self, exa: PackedExa) -> Result<(), PackedExa> {
        let ct = match self.backbone.cap_pool().take_token() {
            Some(ct) => ct,
            None => return Err(exa),
        };
        self.add_exa_internal(exa.hydrate(self.rt.clone()), ct);
        Ok(())
    }

    pub fn collect_incoming_exas(&mut self) {
        let inc = self.backbone.incoming().drain();
        for (exa, ct) in inc {
            self.add_exa_internal(exa.hydrate(self.rt.clone()), ct);
        }
    }

    pub fn step(&mut self) {
        self.collect_incoming_exas();
        if self.exas.is_empty() {
            return;
        }
        let results = self.exec_all();
        self.apply_side_effects(results);
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
                        self.generate_clone(k, j);
                    }
                    _ => (),
                },
                ExaStatus::SideEffect(se) => match se {
                    SideEffect::Kill => self.kill(k),
                    SideEffect::Link(l) => {
                        unimplemented!()
                    }
                },
                ExaStatus::Error(e) => {
                    let name = self.remove_exa_internal(k).0.name;
                    info!("[VM] exa error: {} |> {:?}", name, e);
                }
            }
        }
    }

    fn kill(&mut self, k: usize) {
        let out_len = self.backbone.outgoing().len();
        let total_len = out_len + self.exas.len();
        if total_len < 1 {
            return;
        }
        let kill_active = self
            .rng
            .random_ratio(self.exas.len() as u32, total_len as u32);
        if kill_active {
            let mut kt = self.exas.keys().choose(&mut self.rng).unwrap().to_owned();
            while kt == k {
                kt = self.exas.keys().choose(&mut self.rng).unwrap().to_owned();
            }
            self.remove_exa_internal(kt);
        } else {
            self.backbone.outgoing().kill(&mut self.rng);
        }
    }

    fn generate_clone(&mut self, k: usize, j: u8) {
        let ct = match self.backbone.cap_pool().take_token() {
            Some(ct) => ct,
            None => return,
        };

        let original = self.exas.get_mut(&k).unwrap();
        let mut clone = original.clone();

        original.repl_counter += 1;
        original.instr_ptr += 1;

        clone.repl_counter = 0;
        clone.instr_ptr = j;
        clone.name.push_str(&format!(":{}", original.repl_counter));

        self.add_exa_internal(clone, ct);
    }

    fn add_exa_internal(&mut self, exa: Exa, cap_token: CapacityToken) {
        self.cap_tokens.push(cap_token);
        self.exas.insert(self.get_next_exa_key(), exa);
    }

    fn remove_exa_internal(&mut self, k: usize) -> (Exa, CapacityToken) {
        (
            self.exas.remove(&k).unwrap(),
            self.cap_tokens.pop().unwrap(),
        )
    }

    fn get_next_exa_key(&self) -> usize {
        self.exas.keys().max().unwrap_or(&0) + 1
    }
}
