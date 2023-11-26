mod exa;

use exa::Exa;

fn main() {
    let inst: Vec<String> = vec![
        "copy 5 x",
        "mark LOOP",
        "subi x 1 x",
        "prnt x",
        "test x = 0",
        "fjmp LOOP",
        "prnt 'Loop finished'"
    ].into_iter().map(|s| s.to_string()).collect();

    let fibonacci: Vec<String> = vec![
        "copy 1 t",
        "mark LOOP",
        "prnt x",
        "addi x t x",
        "prnt t",
        "addi x t t",
        "jump LOOP",
    ].into_iter().map(|s| s.to_string()).collect();

    let mut xa: Exa = Exa::new(inst).unwrap();
    xa.start();

    let ser = serde_json::to_string(&xa).unwrap();
    println!("{}", ser);
    let des: Exa = serde_json::from_str(&ser).unwrap();
    println!("{:?}", des);
}
