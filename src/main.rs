mod exa;
mod linker;

use exa::{Exa, ExaStatus};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

fn main() {
    let inst: Vec<String> = vec![
        "copy 10 x",
        "mark LOOP",
        "subi x 1 x",
        "prnt x",
        "test x = 5",
        "tjmp LINK",
        "test x = 0",
        "fjmp LOOP",
        "prnt 'Loop finished'",
        "halt",
        "mark LINK",
        "link 1",
        "jump LOOP",
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

    link_manager.connect("192.168.203.50:6800".to_string());
    
    let result = Exa::new("XA".to_string(), inst);
    
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
    
    match run_exa(&mut xa) {
        ExaStatus::LinkRq(l) => {
            let packed_exa = serde_json::to_string(&xa).unwrap();
            link_manager.send(l.try_into().unwrap(), packed_exa);
        },
        _ => (),
    }
    
    let recieved = return_rx.recv().unwrap();
    println!("recieved exa from foreign host");
    let mut exa: Exa = serde_json::from_str(&recieved).unwrap();
    run_exa(&mut exa);

    handle.join().unwrap();
}

fn run_exa(exa: &mut Exa) -> ExaStatus {
    loop {
        match exa.exec() {
            ExaStatus::Err(e) => return ExaStatus::Err(e),
            ExaStatus::Halt => return ExaStatus::Halt,
            ExaStatus::LinkRq(l) => return ExaStatus::LinkRq(l),
            ExaStatus::Ok => (),
        }
    }
}
