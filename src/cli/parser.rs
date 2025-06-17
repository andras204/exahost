use clap::Parser;

use super::command::*;

/// Interactive parser
#[derive(Debug, Parser)]
#[command(version)]
#[command(long_about=None)]
#[command(multicall = true)]
pub struct Interactive {
    #[command(subcommand)]
    pub command: Command,
}
