use exahost::exavm::VM;
use exahost::file::File;
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
                // "prnt 'Fibonacci'",
                // "copy 1 t",
                // "mark fib",
                // "prnt x",
                // "addi x t t",
                // "prnt t",
                // "addi x t x",
                // "jump fib",
                "copy m x",
                "@rep 5",
                "test x = @{1,1}",
                "tjmp CASE@{1,1}",
                "@end",
                "halt",
                "@rep 5",
                "mark CASE@{1,1}",
                "prnt @{1,1}",
                "halt",
                "@end",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
        )
        .unwrap();
    let xb = rhizome
        .compile_exa(
            "XB",
            vec!["rand 1 5 x", "prnt x", "copy x m"]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        )
        .unwrap();

    let mut vm = VM::new();

    let f = File::from(vec!["asd", "123", "fgh", "456"]);

    vm.add_file(f);

    vm.add_exa(xa);
    vm.add_exa(xb);

    for _ in 0..50 {
        vm.step();
    }

    // let mut stream = TcpStream::connect("localhost:9800").unwrap();
    // println!("dropping connection");
    // drop(stream);

    // asd.join().unwrap();

    // thread::sleep(Duration::from_millis(1000));
}
