use exahost::Host;

fn main() {
    simplelog::TermLogger::init(
        log::LevelFilter::Trace,
        simplelog::Config::default(),
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();

    let mut host = Host::new("Rhizome", 16, &"0.0.0.0:6800", 1);
    let _ = host.start();
}
