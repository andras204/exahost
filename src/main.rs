use exa::Exa;

mod exa;

fn main() {
    let inst: Vec<String> = vec![
        "copy 5 t",
        ";; semicolon comment",
        "mark LOOP",
        ";;a",
        "subi t 1 t",
        "prnt t",
        "note note comment",
        "tjmp LOOP",
        "prnt 'Loop finished'"
    ].into_iter().map(|s| s.to_string()).collect();

    let mut xa: Exa = Exa::new(inst).unwrap();
    xa.start();
}
