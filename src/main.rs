use exahost::file::File;
use exahost::Host;

fn main() {
    let mut rhizome = Host::default();
    rhizome.save_config().unwrap();

    let res = rhizome.compile_exa(
        "ASD",
        vec![
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
        ],
    );

    match res {
        Ok(_) => {
            println!("compiled successfully...");
        }
        Err(errs) => {
            for e in errs {
                eprintln!("{}", e);
            }
        }
    }

    let xa = rhizome
        .compile_exa(
            "XA",
            vec![
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
            ],
        )
        .unwrap();
    let xb = rhizome
        .compile_exa(
            "XB",
            vec![
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
            ],
        )
        .unwrap();

    let xc = rhizome.compile_exa("XC", vec!["host x", "prnt x"]).unwrap();

    let fi = rhizome
        .compile_exa(
            "FI",
            vec![
                "copy 1 t",
                "mark LOOP",
                "prnt x",
                "addi x t t",
                "prnt t",
                "addi x t x",
                "jump LOOP",
            ],
        )
        .unwrap();

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
