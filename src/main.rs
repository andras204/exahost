use std::{thread, time::Duration};

use exahost::{
    Host,
    exa::Exa,
};

fn main() {
    let mut rizhome = Host::new("Rizhome", "localhost:6800");
    rizhome.connect("localhost:6800");
    let xa = Exa::new("XA", vec![
        "link 800",
        "prnt 'travelled'",
    ].into_iter().map(|s| s.to_string()).collect()).unwrap();
    let xb = Exa::new("XB", vec![
        "link 800",
        "prnt 'travelled'",
    ].into_iter().map(|s| s.to_string()).collect()).unwrap();
    
    rizhome.add_exa(xa);
    rizhome.add_exa(xb);

    for _ in 0..3 {
        rizhome.step();
        thread::sleep(Duration::from_millis(250));
        println!("--------------------------------------------");
    }
}
