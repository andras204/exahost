use std::{net::SocketAddr, path::PathBuf};

use clap::{Parser, Subcommand};

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
pub struct CliParser {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Command {
    /// Start continous VM execution
    Run,
    /// Execute one VM cycle
    Step,
    /// Stop VM execution
    Stop,
    /// Kill all EXAs in VM
    KillAll,
    /// Connect to another exahost instane
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
    /// Disconnect from all other exahost instances
    DisconnectAll,
    /// Compile exa to file
    #[command(arg_required_else_help = true)]
    Compile {
        input_file: PathBuf,
        output_file: Option<PathBuf>,
    },
    /// Load exa binaries
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
    /// Shut down all components and exit
    Exit,
}
