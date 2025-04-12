use std::sync::{
    atomic::{
        AtomicUsize,
        Ordering::{Acquire, Release},
    },
    Arc, Weak,
};

#[derive(Debug)]
struct CapacityPoolInner {
    used_capacity: AtomicUsize,
    max_capacity: AtomicUsize,
}

#[derive(Debug, Clone)]
pub struct CapacityPool {
    inner: Arc<CapacityPoolInner>,
}

#[derive(Debug)]
pub struct CapacityToken {
    pool_ref: Weak<CapacityPoolInner>,
}

impl CapacityToken {
    fn generate_new(cap_pool_inner: &Arc<CapacityPoolInner>) -> Self {
        cap_pool_inner.used_capacity.store(
            cap_pool_inner.used_capacity.load(Acquire).saturating_add(1),
            Release,
        );
        Self {
            pool_ref: Arc::downgrade(cap_pool_inner),
        }
    }
}

impl Drop for CapacityToken {
    fn drop(&mut self) {
        let pool_ref = self.pool_ref.upgrade().unwrap();
        pool_ref.used_capacity.store(
            pool_ref.used_capacity.load(Acquire).saturating_sub(1),
            Release,
        );
    }
}

impl CapacityPool {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            inner: Arc::new(CapacityPoolInner {
                used_capacity: AtomicUsize::new(0),
                max_capacity: AtomicUsize::new(max_capacity),
            }),
        }
    }

    pub fn update_max_capacity(&self, new_cap: usize) {
        self.inner.max_capacity.store(new_cap, Release);
    }

    pub fn max_capacity(&self) -> usize {
        self.inner.max_capacity.load(Acquire)
    }

    pub fn used(&self) -> usize {
        self.inner.used_capacity.load(Acquire)
    }

    pub fn free(&self) -> usize {
        self.max_capacity().saturating_sub(self.used())
    }

    pub fn has_free(&self) -> bool {
        self.free() > 0
    }

    pub fn take_token(&self) -> Option<CapacityToken> {
        if self.used() < self.max_capacity() {
            Some(CapacityToken::generate_new(&self.inner))
        } else {
            None
        }
    }
}
