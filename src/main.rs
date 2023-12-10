use exahost::{Host, exavm::exa::Exa};

fn main() {
    let mut rizhome = Host::new("Rizhome", "localhost:6800");
    let exa = Exa::new("XA".to_string(), vec!["prnt 'asd'".to_string()]).unwrap();
    rizhome.add_exa(exa);
    rizhome.step();
}
