use std::collections::HashMap;
use flume::{Sender, Receiver};
use std::thread::{self, JoinHandle};
use std::net::*;
use std::io::{BufReader, LineWriter, BufRead, Write};

use crate::signal::HostSignal;

pub struct LinkManager {
    bind_addr: String,
    retrun_tx: Sender<String>,
    links: HashMap<u16, Sender<String>>,
}

impl LinkManager {
    pub fn new(bind_addr: String, retrun_tx: Sender<String>) -> LinkManager {
        LinkManager {
            bind_addr,
            retrun_tx,
            links: HashMap::new(),
        }
    }

    pub fn start_listening(&self) {
        let tx_clone = self.retrun_tx.clone();
        let addr_clone = self.bind_addr.clone();
        //thread::spawn(|| Self::listen(addr_clone, tx_clone))
    }

    pub fn connect(&mut self, addr: String) {
        let stream = TcpStream::connect(addr).unwrap();
        let (tx, rx) = flume::unbounded();
        self.links.insert(1, tx);
        thread::spawn(move || Self::handle_outgoing(stream, rx));

    }

    pub fn send(&self, link: u16, msg: String) {
        self.links.get(&link).unwrap().send(msg).unwrap();
    }

    fn listen(bind_addr: String, retrun_tx: Sender<String>) {
        let listener = TcpListener::bind(bind_addr).unwrap();
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("accepted connection from {}", stream.peer_addr().unwrap());
                    let tx_clone = retrun_tx.clone();
                    thread::spawn(move || Self::handle_incoming(stream, tx_clone));
                },
                Err(e) => eprintln!("{}", e),
            }
        }
    }

    fn handle_incoming(stream: TcpStream, retrun_tx: Sender<String>) {
        loop {
            let mut line = String::new();
            let mut reader = BufReader::new(&stream);
            let bytes_read = reader.read_line(&mut line).unwrap();
            if bytes_read == 0 { break; }
            retrun_tx.send(line.trim().to_string()).unwrap();
        }
    }

    fn handle_outgoing(stream: TcpStream, receiver: Receiver<String>) {
        loop {
            let mut msg = receiver.recv().unwrap();
            msg.push('\n');
            let mut writer = LineWriter::new(&stream);
            writer.write_all(msg.as_bytes()).unwrap();
        }
    }
}
