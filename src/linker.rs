use std::collections::HashMap;
use std::thread;
use std::net::*;
use std::io::{BufReader, LineWriter, BufRead, Write};
use std::sync::{Arc, Mutex};
use flume::{Sender, Receiver};
use serde::{Serialize, Deserialize};

use crate::exa::Exa;
use crate::signal::HostSignal;

#[derive(Debug)]
struct Connection {
    stream: TcpStream,
    writer: LineWriter<TcpStream>,
    tx: Sender<HostSignal>,
    link: i16,
}

impl Connection {
    pub fn new(stream: TcpStream, tx: Sender<HostSignal>, link: i16) -> Self {
        Connection {
            stream: stream.try_clone().unwrap(),
            writer: LineWriter::new(stream),
            tx,
            link,
        }
    }

    pub fn send<'a>(&mut self, item: &(impl Serialize + Deserialize<'a>)) {
        let mut msg = serde_json::to_string(item).unwrap();
        msg.push(char::from_u32(0xA).unwrap());
        self.writer.write_all(msg.as_bytes()).unwrap();
        println!("[linker]: sent exa to link-{}", self.link);
    }

    pub fn start_read_loop(&self) {
        let t = self.tx.clone();
        let s = self.stream.try_clone().unwrap();
        let l = self.link.clone();
        thread::spawn(move || {
            Self::listen_loop(s, t, l);
        });
    }

    fn listen_loop(stream: TcpStream, tx: Sender<HostSignal>, link: i16) {
        let mut reader = BufReader::new(stream);
        println!("[linker]: listen loop for link-{} started", link);
        loop {
            let mut buf = String::new();
            println!("[conn | {}]: buffer created", link);
            let bytes = reader.read_line(&mut buf).unwrap();
            println!("[conn | {}]: bytes read = {}", link, bytes);
            if bytes == 0 { break; }
            //let exa: Exa = serde_json::from_str(&buf).unwrap();
            println!("[conn | {}]: deserialized", link);
            //tx.send(HostSignal::Link((link, exa))).unwrap();
            println!("[linker]: recieved exa from link-{}", link);
        }
    }
}

pub struct LinkManager {
    bind_addr: SocketAddr,
    links: Arc<Mutex<HashMap<i16, Connection>>>,
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
            links: Arc::new(Mutex::new(HashMap::new())),
            incoming_tx,
            incoming_rx,
            outgoing_tx,
            outgoing_rx,
        }
    }

    pub fn start_listening(&mut self) -> (Sender<HostSignal>, Receiver<HostSignal>) {
        let addr = self.bind_addr.clone();
        let links_ref = self.links.clone();
        let tx = self.outgoing_tx.clone();
        thread::spawn(move || {
            Self::listen_loop(addr, links_ref, tx);
        });
        let links_ref = self.links.clone();
        let rx = self.incoming_rx.clone();
        thread::spawn(move || {
            Self::send_loop(links_ref, rx);
        });
        (self.incoming_tx.clone(), self.outgoing_rx.clone())
    }

    pub fn connect(&mut self, address: &(impl ToSocketAddrs + ?Sized)) {
        let addr = address.to_socket_addrs().unwrap().last().unwrap();
        let stream = TcpStream::connect(addr).unwrap();
        let mut links = self.links.lock().unwrap();
        let link_id = match links.keys().max() {
            Some(n) => n.to_owned() + 1,
            None => 800,
        };
        let con = Connection::new(stream, self.outgoing_tx.clone(), link_id);
        con.start_read_loop();
        println!("[linker]: connecting to {}, enumerated as link-{}",
            addr,
            link_id,
        );
        links.insert(link_id as i16, con);
    }

    fn listen_loop(addr: SocketAddr, links: Arc<Mutex<HashMap<i16, Connection>>>, tx: Sender<HostSignal>) {
        let listener = TcpListener::bind(addr).unwrap();
        println!("[linker]: listening on {}", listener.local_addr().unwrap());
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let mut links = links.lock().unwrap();
            let link_id = match links.keys().max() {
                Some(n) => n.to_owned() + 1,
                None => 800,
            };
            let con = Connection::new(stream, tx.clone(), link_id);
            println!("[linker]: new connection from {}, enumerated as link-{}",
                con.stream.peer_addr().unwrap(),
                link_id,
            );
            links.insert(link_id as i16, con);
            drop(links);
        }
    }

    fn send_loop(links: Arc<Mutex<HashMap<i16, Connection>>>, rx: Receiver<HostSignal>) {
        for msg in rx.iter() {
            match msg {
                HostSignal::Link(link) => {
                    let mut links = links.lock().unwrap();
                    links.get_mut(&link.0).unwrap().send(&link.1);
                    drop(links);
                },
                _ => (),
            }
        }
    }
}
