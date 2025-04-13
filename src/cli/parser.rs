use clap::Parser;

use super::command::*;

/// Startup parser
#[derive(Debug, Parser)]
#[command(version)]
#[command(long_about=None)]
pub struct Startup {
    #[command(subcommand)]
    pub command: StartCommand,
}

/// Interactive parser
#[derive(Debug, Parser)]
#[command(version)]
#[command(long_about=None)]
pub struct Interactive {
    #[command(subcommand)]
    pub command: Command,
}
