use std::collections::{hash_map::Keys, HashMap};

use crate::exa::PackedExa;

#[derive(Debug)]
pub struct VMBridge {
    outgoing: HashMap<usize, (i16, PackedExa)>,
    incoming: Vec<PackedExa>,
    max_capacity: usize,
    current_capacity: usize,
}

impl VMBridge {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            outgoing: HashMap::new(),
            incoming: Vec::new(),
            max_capacity,
            current_capacity: 0,
        }
    }

    pub fn add_outgoing(&mut self, k: usize, link: i16, pexa: PackedExa) {
        self.current_capacity += 1;
        self.outgoing.insert(k, (link, pexa));
    }

    pub fn txfer_to_outgoing(&mut self, k: usize, link: i16, pexa: PackedExa) {
        self.outgoing.insert(k, (link, pexa));
    }

    pub fn remove_outgoing(&mut self, k: &usize) -> Option<(usize, (i16, PackedExa))> {
        self.current_capacity -= 1;
        self.outgoing.remove_entry(k)
    }

    pub fn txfer_from_outgoing(&mut self, k: &usize) -> Option<(usize, (i16, PackedExa))> {
        self.outgoing.remove_entry(k)
    }

    pub fn keys(&self) -> Keys<'_, usize, (i16, PackedExa)> {
        self.outgoing.keys()
    }

    pub fn push_incoming(&mut self, pexa: PackedExa) {
        self.current_capacity += 1;
        self.incoming.push(pexa);
    }

    pub fn collect_incoming(&mut self) -> Vec<PackedExa> {
        self.current_capacity -= self.incoming.len();
        self.incoming.drain(..).collect()
    }

    pub fn txfer_from_incoming(&mut self) -> Vec<PackedExa> {
        self.incoming.drain(..).collect()
    }

    pub fn update_capacity(&mut self, vm_cap: usize) {
        self.current_capacity = self.outgoing.len() + self.incoming.len() + vm_cap;
    }

    pub fn outgoing_len(&self) -> usize {
        self.outgoing.len()
    }

    pub fn is_full(&self) -> bool {
        self.current_capacity >= self.max_capacity
    }

    pub fn has_space(&self) -> bool {
        !self.is_full()
    }
}
