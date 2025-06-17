use std::{net::SocketAddr, sync::Arc};

use flume::{Receiver, Sender};
use tokio::sync::{
    broadcast::{
        channel as broadcast_channel, Receiver as BroadcastReciever, Sender as BroadcastSender,
    },
    Mutex,
};

use capacity_pool::CapacityPool;
use connection_store::ConnectionStore;
use exa_buffers::{ExaInBuffer, ExaOutBuffer};
use server_command::ServerCommand;
use vm_command::VMCommand;

use crate::compiler::Compiler;

pub mod capacity_pool;
mod connection_store;
mod exa_buffers;
pub mod server_command;
pub mod vm_command;

#[derive(Debug, Clone)]
pub struct Backbone {
    cap_pool: CapacityPool,
    exa_in_buffer: ExaInBuffer,
    exa_out_buffer: ExaOutBuffer,
    connections: ConnectionStore,

    compiler: Arc<Compiler>,

    hostname: Box<str>,
    server_listening_addr: Arc<Mutex<Option<SocketAddr>>>,

    shutdown_broadcast: BroadcastSender<()>,
    server_control_tx: Sender<ServerCommand>,
    server_control_rx: Receiver<ServerCommand>,
    vm_control_tx: Sender<VMCommand>,
    vm_control_rx: Receiver<VMCommand>,
}

impl Backbone {
    pub fn new(hostname: &str, max_capacity: usize, compiler: Compiler) -> Self {
        let (shutdown_tx, _) = broadcast_channel(1);
        let (server_tx, server_rx) = flume::unbounded();
        let (vm_tx, vm_rx) = flume::unbounded();
        Self {
            cap_pool: CapacityPool::new(max_capacity),
            exa_in_buffer: ExaInBuffer::new(),
            exa_out_buffer: ExaOutBuffer::new(),
            connections: ConnectionStore::new(),
            compiler: Arc::new(compiler),
            hostname: hostname.into(),
            server_listening_addr: Arc::new(Mutex::new(None)),
            shutdown_broadcast: shutdown_tx,
            server_control_tx: server_tx,
            server_control_rx: server_rx,
            vm_control_tx: vm_tx,
            vm_control_rx: vm_rx,
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

    pub fn compiler(&self) -> &Compiler {
        &self.compiler
    }

    pub fn hostname(&self) -> &Box<str> {
        &self.hostname
    }

    pub fn send_shutdown_signal(&self) {
        self.shutdown_broadcast.send(()).unwrap();
    }

    pub fn get_shutdown_listener(&self) -> BroadcastReciever<()> {
        self.shutdown_broadcast.subscribe()
    }

    pub fn send_server_command(&self, command: ServerCommand) {
        self.server_control_tx.send(command).unwrap();
    }

    pub fn server_command_rx(&self) -> &Receiver<ServerCommand> {
        &self.server_control_rx
    }

    pub fn send_vm_command(&self, command: VMCommand) {
        self.vm_control_tx.send(command).unwrap();
    }

    pub fn vm_control_rx(&self) -> &Receiver<VMCommand> {
        &self.vm_control_rx
    }

    pub fn set_server_listening_addr(&self, addr: SocketAddr) {
        *self.server_listening_addr.blocking_lock() = Some(addr);
    }

    pub async fn get_server_listening_addr_async(&self) -> SocketAddr {
        self.server_listening_addr.lock().await.clone().unwrap()
    }
}
