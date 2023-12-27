use std::{thread, time::Duration};

use exahost::{
    Host,
    exa::Exa,
};

fn main() {
    let mut rhizome = Host::new("Rhizome", "localhost:6800");
    //rhizome.connect("localhost:6800");
    let xa = Exa::new("XA", vec![
        "copy 10 m",
    ].into_iter().map(|s| s.to_string()).collect()).unwrap();
    let xb = Exa::new("XB", vec![
        "prnt m",
    ].into_iter().map(|s| s.to_string()).collect()).unwrap();
    
    rhizome.add_exa(xa);
    rhizome.add_exa(xb);

    for _ in 0..3 {
        rhizome.step();
        thread::sleep(Duration::from_millis(250));
    }
}
