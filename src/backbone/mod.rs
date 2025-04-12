use std::{net::SocketAddr, sync::Arc};

use capacity_pool::CapacityPool;
use connection_store::ConnectionStore;
use exa_buffers::{ExaInBuffer, ExaOutBuffer};
use flume::{Receiver, Sender};
use tokio::sync::{
    broadcast::{
        channel as broadcast_channel, Receiver as BroadcastReciever, Sender as BroadcastSender,
    },
    Mutex,
};

use crate::message::ServerCommand;

pub mod capacity_pool;
mod connection_store;
mod exa_buffers;

#[derive(Debug, Clone)]
pub struct Backbone {
    cap_pool: CapacityPool,
    exa_in_buffer: ExaInBuffer,
    exa_out_buffer: ExaOutBuffer,
    connections: ConnectionStore,

    server_listening_addr: Arc<Mutex<Option<SocketAddr>>>,

    shutdown_broadcast: BroadcastSender<()>,
    server_control_tx: Sender<ServerCommand>,
    server_control_rx: Receiver<ServerCommand>,
}

impl Backbone {
    pub fn new(max_capacity: usize) -> Self {
        let (shutdown_tx, _) = broadcast_channel(1);
        let (server_tx, server_rx) = flume::unbounded();
        Self {
            cap_pool: CapacityPool::new(max_capacity),
            exa_in_buffer: ExaInBuffer::new(),
            exa_out_buffer: ExaOutBuffer::new(),
            connections: ConnectionStore::new(),
            server_listening_addr: Arc::new(Mutex::new(None)),
            shutdown_broadcast: shutdown_tx,
            server_control_tx: server_tx,
            server_control_rx: server_rx,
        }
    }

    pub fn cap_pool(&self) -> &CapacityPool {
        &self.cap_pool
    }

    pub fn incoming(&self) -> &ExaInBuffer {
        &self.exa_in_buffer
    }

    pub fn outgoing(&self) -> &ExaOutBuffer {
        &self.exa_out_buffer
    }

    pub fn connections(&self) -> &ConnectionStore {
        &self.connections
    }

    pub fn send_shutdown_signal(&self) {
        self.shutdown_broadcast.send(()).unwrap();
    }

    pub fn get_shutdown_listener(&self) -> BroadcastReciever<()> {
        self.shutdown_broadcast.subscribe()
    }

    pub fn send_server_command(&self, sc: ServerCommand) {
        self.server_control_tx.send(sc).unwrap();
    }

    pub fn server_command_rx(&self) -> &Receiver<ServerCommand> {
        &self.server_control_rx
    }

    pub async fn set_server_listening_addr(&self, addr: SocketAddr) {
        *self.server_listening_addr.lock().await = Some(addr);
    }

    pub async fn get_server_listening_addr(&self) -> SocketAddr {
        self.server_listening_addr.lock().await.clone().unwrap()
    }
}
