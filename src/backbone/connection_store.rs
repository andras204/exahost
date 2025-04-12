use std::{net::SocketAddr, sync::Arc};

use nohash_hasher::IntMap;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct ConnectionStore {
    connections: Arc<Mutex<IntMap<i16, SocketAddr>>>,
}

impl ConnectionStore {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(IntMap::default())),
        }
    }

    pub fn add(&self, link_id: i16, addr: SocketAddr) {
        self.connections.blocking_lock().insert(link_id, addr);
    }

    pub async fn add_async(&self, link_id: i16, addr: SocketAddr) {
        self.connections.lock().await.insert(link_id, addr);
    }

    pub fn remove(&self, link_id: i16) {
        self.connections.blocking_lock().remove(&link_id);
    }

    pub async fn remove_async(&self, link_id: i16) {
        self.connections.lock().await.remove(&link_id);
    }

    pub fn get_addr(&self, link_id: i16) -> Option<SocketAddr> {
        match self.connections.blocking_lock().get(&link_id) {
            Some(a) => Some(a.to_owned()),
            None => None,
        }
    }

    pub async fn get_addr_async(&self, link_id: i16) -> Option<SocketAddr> {
        match self.connections.lock().await.get(&link_id) {
            Some(a) => Some(a.to_owned()),
            None => None,
        }
    }

    pub fn auto_enum(&self, incoming: bool) -> i16 {
        if incoming {
            self.connections
                .blocking_lock()
                .keys()
                .min()
                .unwrap_or(&0)
                .to_owned()
                - 1
        } else {
            self.connections
                .blocking_lock()
                .keys()
                .max()
                .unwrap_or(&0)
                .to_owned()
                + 1
        }
    }

    pub async fn auto_enum_async(&self, incoming: bool) -> i16 {
        if incoming {
            self.connections
                .lock()
                .await
                .keys()
                .min()
                .unwrap_or(&0)
                .to_owned()
                - 1
        } else {
            self.connections
                .lock()
                .await
                .keys()
                .max()
                .unwrap_or(&0)
                .to_owned()
                + 1
        }
    }
}
