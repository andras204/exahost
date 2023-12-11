use exahost::{
    Host,
    exa::Exa
};

fn main() {
    let mut rizhome = Host::new("Rizhome", "localhost:6800");
    let xa = Exa::new("XA".to_string(), vec![
        "mark LOOP",
        "addi 1 x x",
        "repl ASD",
        "jump LOOP",
        "mark ASD",
        "prnt 'Im a clone!'",
        "prnt x",
    ].into_iter().map(|s| s.to_string()).collect()).unwrap();

    let xb = Exa::new("XB".to_string(), vec![
        "copy 10 t",
        "mark LOOP",
        "subi t 1 t",
        "tjmp LOOP",
        "prnt 'killing xa'",
        "kill",
        "halt",
    ].into_iter().map(|s| s.to_string()).collect()).unwrap();

    rizhome.add_exa(xa);
    rizhome.add_exa(xb);

    for x in 0..40 {
        println!("step {}:", x);
        rizhome.step();
    }
}
