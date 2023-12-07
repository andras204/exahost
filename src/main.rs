mod exa;
mod linker;

use exa::Exa;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

fn main() {
    let inst: Vec<String> = vec![
        "copy 5 x",
        "mark LOOP",
        "subi x 1 x",
        "prnt x",
        "test x = 0",
        "fjmp LOOP",
        "prnt 'Loop finished'"
    ].into_iter().map(|s| s.to_string()).collect();

    let fibonacci: Vec<String> = vec![
        "copy 1 t",
        "mark LOOP",
        "prnt x",
        "addi x t x",
        "prnt t",
        "addi x t t",
        "jump LOOP",
    ].into_iter().map(|s| s.to_string()).collect();

    let (return_tx, return_rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    println!("starting TCP listener...");
    let mut link_manager = linker::LinkManager::new("localhost:6800".to_string(), return_tx.clone());
    let handle = link_manager.start_listening();
    let addr = return_rx.recv().unwrap();
    println!("listening on address: {}", addr);

    link_manager.connect("localhost:6800".to_string());

    let result = Exa::new("XA".to_string(), vec!["halt".to_string()]);
    
    let mut xa: Exa;
    
    match result {
        Ok(e) => xa = e,
        Err(errs) => {
            for e in errs {
                eprintln!("{}", e);
            }
            panic!();
        },
    }
    
    for _ in 0..30 {
        xa.exec().unwrap();
    }

    let packed_exa = serde_json::to_string(&xa).unwrap();

    link_manager.send(1, packed_exa);

    let recieved = return_rx.recv().unwrap();
    println!("recieved packed exa:");
    let mut xb: Exa = serde_json::from_str(&recieved[..]).unwrap();
    println!("{:?}", xb);

    for _ in 0..30 {
        xb.exec().unwrap();
    }

    handle.join().unwrap();
}
