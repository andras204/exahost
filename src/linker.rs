use std::collections::HashMap;
use std::io::BufWriter;
use std::thread;
use std::net::*;
use std::io::{BufReader, LineWriter, BufRead, Write};
use flume::{Sender, Receiver};

use crate::exa::Exa;
use crate::signal::HostSignal;

pub struct LinkManager {
    bind_addr: SocketAddr,
    links: HashMap<i16, Sender<Exa>>,
    incoming_tx: Sender<HostSignal>,
    incoming_rx: Receiver<HostSignal>,
    outgoing_tx: Sender<HostSignal>,
    outgoing_rx: Receiver<HostSignal>,
}

impl LinkManager {
    pub fn new(bind_address: &(impl ToSocketAddrs + ?Sized)) -> LinkManager {
        let (incoming_tx, incoming_rx) = flume::unbounded();
        let (outgoing_tx, outgoing_rx) = flume::unbounded();
        let bind_addr = bind_address.to_socket_addrs().unwrap().last().unwrap();
        LinkManager {
            bind_addr,
            links: HashMap::new(),
            incoming_tx,
            incoming_rx,
            outgoing_tx,
            outgoing_rx,
        }
    }

    pub fn start_listening(&mut self) -> (Sender<HostSignal>, Receiver<HostSignal>) {
        let addr = self.bind_addr.clone();
        let tx = self.outgoing_tx.clone();
        thread::spawn(move || {
            Self::listen_loop(addr, tx);
        });
        (self.incoming_tx.clone(), self.outgoing_rx.clone())
    }

    pub fn connect(&mut self, address: &(impl ToSocketAddrs + ?Sized)) {
        let addr = address.to_socket_addrs().unwrap().last().unwrap();
        let stream = TcpStream::connect(addr).unwrap();
        let (tx, rx) = flume::unbounded();
        let link_id = match self.links.keys().max() {
            Some(n) => n + 1,
            None => 800,
        };
        self.links.insert(link_id as i16, tx);
        thread::spawn(move || {
            Self::handle_outgoing(stream, rx)
        });
    }

    pub fn send(&self, link: i16, exa: Exa) -> Result<(), &str> {
        match self.links.get(&link) {
            Some(s) => Ok(s.send(exa).unwrap()),
            None => Err("Invalid link"),
        }
    }

    fn listen_loop(addr: SocketAddr, tx: Sender<HostSignal>) {
        let listener = TcpListener::bind(addr).unwrap();
        for result in listener.incoming() {
            let stream = result.unwrap();
            let t = tx.clone();
            thread::spawn(move || {
                Self::handle_incoming(stream, t);
            });
        }
    }

    fn handle_incoming(stream: TcpStream, tx: Sender<HostSignal>) {
        let mut reader = BufReader::new(stream);
        loop {
            let mut buf = String::new();
            let bytes = reader.read_line(&mut buf).unwrap();
            if bytes == 0 { return; }
            let exa: Exa = serde_json::from_str(&buf).unwrap();
            tx.send(HostSignal::Link((0, exa))).unwrap();
        }
    }

    fn handle_outgoing(stream: TcpStream, rx: Receiver<Exa>) {
        let mut writer = LineWriter::new(stream);
        loop {
            let exa = match rx.recv() {
                Ok(e) => e,
                Err(_) => return,
            };
            let mut msg = serde_json::to_string(&exa).unwrap();
            msg.push('\n');
            writer.write_all(msg.as_bytes()).unwrap();
        }
    }
}
