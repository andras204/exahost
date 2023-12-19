use exahost::{
    Host,
    exa::Exa,
};

fn main() {
    let mut rizhome = Host::new("Rizhome", "localhost:6800");
    let xa = Exa::new("XA", vec![
        "copy 10 m",
        "prnt m",
        "copy 10 m",
    ].into_iter().map(|s| s.to_string()).collect()).unwrap();

    let xb = Exa::new("XB", vec![
        "prnt m",
        "copy 10 m",
        "prnt m",
    ].into_iter().map(|s| s.to_string()).collect()).unwrap();

    rizhome.add_exa(xa);
    rizhome.add_exa(xb);

    for _ in 0..10 {
        rizhome.step();
    }
}
