use exahost::file::File;
use exahost::Host;

fn main() {
    let mut rhizome = Host::default();
    rhizome.save_config().unwrap();

    let test = vec![
        "@rep 5",
        "addi 32000 x t",
        "@end",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "prnt 'asdasfsdgsfg'",
    ];

    let switch = vec![
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
    ];

    let reader = vec![
        "grab 0",
        "mark ASD",
        "seek -999",
        "rand 1 5 x",
        "prnt x",
        "seek x",
        "test eof",
        "tjmp ASD",
        "copy f x",
        "prnt x",
        "copy x m",
    ];

    // let fibonacci = vec![
    //     "copy 1 t",
    //     "mark LOOP",
    //     "prnt x",
    //     "addi x t t",
    //     "prnt t",
    //     "addi x t x",
    //     "jump LOOP",
    // ];
    let fibonacci = vec!["muli 9999 9999 x", "prnt x"];

    let res = rhizome.compile_exa("ASD", test);

    match res {
        Ok(_) => {
            println!("compiled successfully...");
        }
        Err(errs) => {
            for e in errs {
                eprintln!("{:?}", e);
            }
        }
    }

    let xa = rhizome.compile_exa("XA", switch).unwrap();
    let xb = rhizome.compile_exa("XB", reader).unwrap();

    let xc = rhizome.compile_exa("XC", vec!["host x", "prnt x"]).unwrap();

    let fi = rhizome.compile_exa("FI", fibonacci).unwrap();

    let f = File::from(vec!["1", "2", "3", "4", "5"]);

    rhizome.add_file(f);

    // rhizome.add_exa(xa);
    // rhizome.add_exa(xb);
    // rhizome.add_exa(xc);
    rhizome.add_exa(fi);

    for _ in 0..70 {
        rhizome.step();
    }
}
