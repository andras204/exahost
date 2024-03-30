use exahost::exavm::VM;
use exahost::Host;

fn main() {
    let mut rhizome = Host::new("Rhizome", "localhost:6800");
    // rhizome.connect("localhost:6800");

    // let asd = thread::spawn(|| {
    //     let lm = LinkManager::new();
    //     lm.start_listening("0.0.0.0:9800");
    // });

    let xa = rhizome
        .compile_exa(
            "XA",
            vec![
                "prnt 'Fibonacci'",
                "copy 1 t",
                "mark fib",
                "prnt x",
                "addi x t t",
                "prnt t",
                "addi x t x",
                "jump fib",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
        )
        .unwrap();

    let mut vm = VM::new();

    vm.add_exa(xa);

    for _ in 0..50 {
        vm.step();
    }

    // let mut stream = TcpStream::connect("localhost:9800").unwrap();
    // println!("dropping connection");
    // drop(stream);

    // asd.join().unwrap();

    // thread::sleep(Duration::from_millis(1000));
}
