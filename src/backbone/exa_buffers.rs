use std::sync::Arc;

use nohash_hasher::{IntMap, IntSet};
use rand::{rngs::ThreadRng, seq::IteratorRandom};
use tokio::sync::Mutex;

use crate::exa::PackedExa;

use super::capacity_pool::CapacityToken;

#[derive(Debug, Clone)]
pub struct ExaOutBuffer {
    inner: Arc<Mutex<Inner>>,
}

#[derive(Debug)]
struct Inner {
    exas: IntMap<usize, PackedExa>,
    links: IntMap<usize, i16>,
    handled: IntSet<usize>,
    tokens: Vec<CapacityToken>,
}

impl ExaOutBuffer {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                exas: IntMap::default(),
                links: IntMap::default(),
                handled: IntSet::default(),
                tokens: Vec::new(),
            })),
        }
    }

    pub fn store_exa(&self, (k, exa): (usize, PackedExa), link_id: i16, t: CapacityToken) {
        let mut inner = self.inner.blocking_lock();
        inner.exas.insert(k, exa);
        inner.links.insert(k, link_id);
        inner.tokens.push(t);
    }

    pub fn take_exa(&self, k: usize) -> Option<(PackedExa, CapacityToken)> {
        let mut inner = self.inner.blocking_lock();
        inner.handled.remove(&k);
        inner.links.remove(&k);
        Some((inner.exas.remove(&k)?, inner.tokens.pop().unwrap()))
    }

    pub async fn take_exa_async(&self, k: usize) -> Option<(PackedExa, CapacityToken)> {
        let mut inner = self.inner.lock().await;
        inner.handled.remove(&k);
        inner.links.remove(&k);
        Some((inner.exas.remove(&k)?, inner.tokens.pop().unwrap()))
    }

    pub async fn mark_handled_async(&self, k: usize) {
        let mut inner = self.inner.lock().await;
        if inner.exas.contains_key(&k) {
            inner.handled.insert(k);
        }
    }

    pub async fn mark_unhandled_async(&self, k: usize) {
        self.inner.lock().await.handled.remove(&k);
    }

    pub fn kill(&self, rng: &mut ThreadRng) {
        let mut inner = self.inner.blocking_lock();
        if inner.exas.is_empty() {
            return;
        }
        let k = inner.exas.keys().choose(rng).unwrap().to_owned();
        inner.handled.remove(&k);
        inner.links.remove(&k);
        inner.exas.remove(&k);
        inner.tokens.pop();
    }

    pub fn kill_all(&self) {
        let mut inner = self.inner.blocking_lock();
        if inner.exas.is_empty() {
            return;
        }
        inner.handled.clear();
        inner.links.clear();
        inner.exas.clear();
        inner.tokens.clear();
    }

    pub async fn get_link_async(&self, exa_id: usize) -> Option<i16> {
        match self.inner.lock().await.links.get(&exa_id) {
            Some(l) => Some(*l),
            None => None,
        }
    }

    pub async fn get_unhandled_keys_async(&self) -> Vec<usize> {
        let inner = self.inner.lock().await;
        inner
            .exas
            .keys()
            .filter_map(|k| {
                if inner.handled.contains(k) {
                    None
                } else {
                    Some(*k)
                }
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.inner.blocking_lock().exas.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.blocking_lock().exas.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct ExaInBuffer {
    buffer: Arc<Mutex<Vec<(PackedExa, CapacityToken)>>>,
}

impl ExaInBuffer {
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn push(&self, exa: PackedExa, t: CapacityToken) {
        self.buffer.blocking_lock().push((exa, t));
    }

    pub async fn push_async(&self, exa: PackedExa, t: CapacityToken) {
        self.buffer.lock().await.push((exa, t));
    }

    pub fn drain(&self) -> Vec<(PackedExa, CapacityToken)> {
        self.buffer.blocking_lock().drain(..).collect()
    }
}
