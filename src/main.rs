use std::{thread, time::Duration};

use exahost::{
    Host,
    exa::Exa,
};

fn main() {
    let mut rhizome = Host::new("Rhizome", "localhost:6800");
    //rhizome.connect("localhost:6800");
    

    for _ in 0..3 {
        rhizome.step();
        thread::sleep(Duration::from_millis(250));
    }
}
