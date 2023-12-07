use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::net::*;
use std::io::{BufReader, LineWriter, BufRead, Write};

pub struct LinkManager {
    bind_addr: String,
    retrun_sender: Sender<String>,
    links: HashMap<u16, Sender<String>>,
}

impl LinkManager {
    pub fn new(bind_addr: String, retrun_sender: Sender<String>) -> LinkManager {
        LinkManager {
            bind_addr,
            retrun_sender,
            links: HashMap::new(),
        }
    }

    pub fn start_listening(&self) -> JoinHandle<()> {
        let sender_clone = self.retrun_sender.clone();
        let addr_clone = self.bind_addr.clone();
        thread::spawn(|| Self::listen(addr_clone, sender_clone))
    }

    pub fn connect(&mut self, addr: String) {
        let stream = TcpStream::connect(addr).unwrap();
        let (tx, rx) = mpsc::channel();
        self.links.insert(1, tx);
        thread::spawn(move || Self::handle_outgoing(stream, rx));

    }

    pub fn send(&self, link: u16, msg: String) {
        self.links.get(&link).unwrap().send(msg).unwrap();
    }

    fn listen(bind_addr: String, retrun_sender: Sender<String>) {
        let listener = TcpListener::bind(bind_addr).unwrap();
        retrun_sender.send(listener.local_addr().unwrap().to_string()).unwrap();
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("accepted connection from {}", stream.peer_addr().unwrap());
                    let sender_clone = retrun_sender.clone();
                    thread::spawn(move || Self::handle_incoming(stream, sender_clone));
                },
                Err(e) => eprintln!("{}", e),
            }
        }
    }

    fn handle_incoming(stream: TcpStream, retrun_sender: Sender<String>) {
        loop {
            let mut line = String::new();
            let mut reader = BufReader::new(&stream);
            let bytes_read = reader.read_line(&mut line).unwrap();
            if bytes_read == 0 { break; }
            retrun_sender.send(line.trim().to_string()).unwrap();
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
