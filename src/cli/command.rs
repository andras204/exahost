use std::{net::SocketAddr, path::PathBuf};

use clap::{Args, Subcommand};

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum StartCommand {
    /// Starts the interactive cli
    Start,
    /// Compile exa to file
    #[command(arg_required_else_help = true)]
    Compile {
        input_file: PathBuf,
        output_file: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Command {
    /// Shut down all components and exit
    Shutdown,
    /// Compile exa to file
    #[command(arg_required_else_help = true)]
    Compile {
        input_file: PathBuf,
        output_file: Option<PathBuf>,
    },
    /// Load exa(s)
    #[command(arg_required_else_help = true)]
    Load {
        /// files to load
        exa_files: Vec<PathBuf>,
    },
    /// Compile and immediately load exa(s)
    #[command(arg_required_else_help = true)]
    CompileLoad {
        /// files to compile and load
        exa_files: Vec<PathBuf>,
    },
    /// Connect to another exahost instance
    #[command(arg_required_else_help = true)]
    Connect {
        /// listening address of instance to connect to
        addr: SocketAddr,
        /// local identifier of the connection, used by LINK instructions
        link_id: Option<i16>,
    },
    /// Disconnect from another exahost instance
    #[command(arg_required_else_help = true)]
    Disconnect { link_id: i16 },
}
