use exahost::{
    compiler::Compiler,
    config::{CompilerConfig, Config},
    exa::PackedExa,
    vm::runtime::Runtime,
};

fn main() {
    let rt = Runtime::new("rhizome", "./files");
    let add_test = vec!["copy 9999 x", "addi 100 x x", "copy x #prnt"];
    let compiler = Compiler::new(CompilerConfig::extended());

    let code = compiler.compile(&add_test[..]).unwrap();
    let mut exa = PackedExa::new("XA", code).hydrate(rt);

    for _ in 0..3 {
        println!("{:?}", exa.exec());
    }

    let config = Config::default();
    let toml = toml::to_string_pretty(&config).unwrap();
    println!("\n{}", toml);
    let dec: Config = toml::from_str(&toml).unwrap();
    print!("\n{:?}", dec);
}
