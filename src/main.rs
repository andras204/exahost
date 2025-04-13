use clap::Parser;
use exahost::cli::parser::{Interactive, Startup};

fn main() {
    simplelog::TermLogger::init(
        log::LevelFilter::Trace,
        simplelog::Config::default(),
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();

    // let start_commands = Startup::parse();
    let _ = Interactive::parse();
}
