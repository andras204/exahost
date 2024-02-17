use exahost::Host;

fn main() {
    let mut rhizome = Host::new("Rhizome", "localhost:6800");
    // rhizome.connect("localhost:6800");

    let xa = rhizome.compile_exa("XA", vec![
        "prnt 'Fibonacci'",
        "copy 1 t",
        "mark fib",
        "prnt x",
        "addi x t t",
        "prnt t",
        "addi x t x",
        "jump fib",
    ].into_iter().map(|s| s.to_string()).collect()).unwrap();

    rhizome.add_exa(xa);
    
    for _ in 0..50 {
        rhizome.step();
    }
}
