use exa::Exa;

mod exa;

fn main() {
    let inst: Vec<String> = vec![
        "copy 5 x",
        "mark LOOP",
        "subi x 1 x",
        "prnt x",
        "prnt t",
        "test x = 0",
        "prnt t",
        "fjmp LOOP",
        "prnt 'Loop finished'"
    ].into_iter().map(|s| s.to_string()).collect();

    let mut xa: Exa = Exa::new(inst);
    xa.start();
}

