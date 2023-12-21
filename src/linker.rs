use std::collections::HashMap;
use std::thread;
use std::sync::{Arc, Mutex};
use std::net::*;
use std::net::ToSocketAddrs;
use std::io::{BufReader, LineWriter, BufRead, Write};
use flume::{Sender, Receiver};
use serde::{Serialize, Deserialize};

use crate::exa::Exa;
use crate::signal::HostSignal;

pub struct Link {
    writer: LineWriter<TcpStream>,
}

impl Link {
    pub fn new(stream: TcpStream, tx: Sender<HostSignal>, link: i16) -> Self {
        let sc = stream.try_clone().unwrap();
        thread::spawn(move || {
            Self::recv_loop(sc, tx, link);
        });
        let writer = LineWriter::new(stream);
        Link {
            writer,
        }
    }

    fn recv_loop(stream: TcpStream, tx: Sender<HostSignal>, link: i16) {
        let mut reader = BufReader::new(stream);
        loop {
            let mut buf = String::new();
            let bytes = reader.read_line(&mut buf).unwrap();
            if bytes == 0 { return; }
            let exa: Exa = serde_json::from_str(&buf).unwrap();
            println!("[linker]: received exa from link-{}", link);
            tx.send(HostSignal::Link((link, exa))).unwrap();
        }
    }

    pub fn send<'a>(&mut self, data: &(impl Serialize + Deserialize<'a>)) {
        let mut msg = serde_json::to_string(&data).unwrap();
        msg.push('\n');
        self.writer.write_all(msg.as_bytes()).unwrap();
    }
}

pub struct LinkManager {
    bind_addr: SocketAddr,
    links: Arc<Mutex<HashMap<i16, Link>>>,
    link_queues: HashMap<i16, Vec<Exa>>,
    tx: Sender<HostSignal>,
    rx: Receiver<HostSignal>,
}

impl LinkManager {
    pub fn new(bind_address: &(impl ToSocketAddrs + ?Sized)) -> LinkManager {
        let (tx, rx) = flume::unbounded();
        let bind_addr = bind_address.to_socket_addrs().unwrap().last().unwrap();
        LinkManager {
            bind_addr,
            links: Arc::new(Mutex::new(HashMap::new())),
            link_queues: HashMap::new(),
            tx,
            rx,
        }
    }

    pub fn connect(&mut self, addr: &(impl ToSocketAddrs + ?Sized)) {
        let stream = TcpStream::connect(addr).unwrap();
        let mut links = self.links.lock().unwrap();
        let link_id = match links.keys().max() {
            Some(n) => n + 1,
            None => 800,
        };
        println!("[linker]: connected to {} | enumerated as link-{}",
                stream.peer_addr().unwrap(), link_id);
        let link = Link::new(stream, self.tx.clone(), link_id.clone());
        links.insert(link_id, link);
    }

    pub fn queue(&mut self, link_rq: HostSignal) -> Result<(), &str> {
        let (l, exa) = match link_rq {
            HostSignal::Link(t) => t,
            _ => return Err("Not link request"),
        };
        let links = self.links.lock().unwrap();
        match links.get(&l) {
            Some(_) => (),
            None => return Err("Invalid link"),
        }
        if self.link_queues.contains_key(&l) {
            self.link_queues.get_mut(&l).unwrap().push(exa);
        }
        else {
            self.link_queues.insert(l, vec![exa]);
        }
        Ok(())
    }

    pub fn send(&mut self) {
        for (k, v) in self.link_queues.iter_mut() {
            let exa = match v.pop() {
                Some(e) => e,
                None => continue,
            };
<<<<<<< HEAD
            let con = Connection::new(stream, tx.clone(), link_id);
            println!("[linker]: new connection from {}, enumerated as link-{}",
                con.stream.peer_addr().unwrap(),
                link_id,
            );
            links.insert(link_id as i16, con);
        }
    }

    fn send_loop(links: Arc<Mutex<HashMap<i16, Connection>>>, rx: Receiver<HostSignal>) {
        for msg in rx.iter() {
            match msg {
                HostSignal::Link(link) => {
                    let mut links = links.lock().unwrap();
                    links.get_mut(&link.0).unwrap().send(&link.1);
                },
                _ => (),
            }
=======
            let mut links = self.links.lock().unwrap();
            links.get_mut(k).unwrap().send(&exa);
            println!("[linker]: sent exa to link-{}", k);
        }
    }

    pub fn start_listening(&mut self) -> Receiver<HostSignal> {
        let addr = self.bind_addr.clone();
        let tx = self.tx.clone();
        let links = self.links.clone();
        thread::spawn(move || {
            Self::listen_loop(addr, tx, links);
        });
        self.rx.clone()
    }

    fn listen_loop(addr: SocketAddr, tx: Sender<HostSignal>, links: Arc<Mutex<HashMap<i16, Link>>>) {
        let listener = TcpListener::bind(addr).unwrap();
        for result in listener.incoming() {
            let stream = result.unwrap();
            let mut links = links.lock().unwrap();
            let link_id = match links.keys().max() {
                Some(n) => n + 1,
                None => 800,
            };
            println!("[linker]: accepted connection from {} | enumerated as {}",
                     stream.peer_addr().unwrap(), link_id);
            let link = Link::new(stream, tx.clone(), link_id.clone());
            links.insert(link_id, link);
>>>>>>> linker
        }
    }
}
