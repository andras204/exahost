use std::{
    io::{BufRead, Read, Write},
    path::PathBuf,
};

use clap::Parser;
use command::Command;
use error::Error;
use parser::Interactive;

use crate::{
    backbone::{server_command::ServerCommand, vm_command::VMCommand, Backbone},
    exa::{instruction::Instruction, PackedExa},
};

pub mod command;
mod error;
pub mod parser;

#[derive(Debug)]
pub struct Cli {
    backbone: Backbone,
    readline_buffer: String,
}

impl Cli {
    pub fn new(backbone: Backbone) -> Self {
        Self {
            backbone,
            readline_buffer: String::with_capacity(2048),
        }
    }

    pub fn start(&mut self) {
        loop {
            let command = match self.parse_line() {
                Ok(c) => c,
                Err(_) => {
                    continue;
                }
            };
            if self.handle_command(command) {
                return;
            }
        }
    }

    fn handle_command(&self, command: Command) -> bool {
        match TryInto::<VMCommand>::try_into(command.clone()) {
            Ok(vmc) => {
                self.backbone.send_vm_command(vmc);
                return false;
            }
            Err(_) => {}
        }

        match TryInto::<ServerCommand>::try_into(command.clone()) {
            Ok(sc) => {
                self.backbone.send_server_command(sc);
                return false;
            }
            Err(_) => {}
        }

        match command {
            Command::Compile {
                input_file,
                output_file,
            } => {
                self.backbone
                    .compiler()
                    .compile_file(&input_file, output_file, true)
                    .unwrap();
                false
            }
            Command::Load { exa_files } => {
                for f in exa_files {
                    let _ = self.try_load_exa(f);
                }
                false
            }
            Command::CompileLoad { exa_files } => {
                for f in exa_files {
                    let instr = self
                        .backbone
                        .compiler()
                        .compile_file(&f, None, false)
                        .unwrap();
                    let _ = self.try_add_exa(&f.file_stem().unwrap().to_string_lossy(), instr);
                }
                false
            }
            Command::Exit => {
                self.backbone.send_shutdown_signal();
                true
            }
            _ => false,
        }
    }

    fn try_load_exa(&self, file_path: PathBuf) -> Result<(), ()> {
        let mut bin = Vec::new();
        match std::fs::File::open(&file_path) {
            Ok(mut b) => match b.read_to_end(&mut bin) {
                Ok(_) => (),
                Err(e) => {
                    println!("error reading {}: {}", file_path.to_string_lossy(), e);
                    return Err(());
                }
            },
            Err(e) => {
                println!("error opening {}: {}", file_path.to_string_lossy(), e);
                return Err(());
            }
        }
        let instr: Box<[Instruction]> = match bitcode::decode(&bin) {
            Ok(i) => i,
            Err(e) => {
                println!("error decoding {}: {}", file_path.to_string_lossy(), e);
                return Err(());
            }
        };
        self.try_add_exa(&file_path.file_stem().unwrap().to_string_lossy(), instr)
    }

    fn try_add_exa(&self, name: &str, instr: Box<[Instruction]>) -> Result<(), ()> {
        let t = match self.backbone.cap_pool().take_token() {
            Some(t) => t,
            None => {
                println!("error loading exa: VM full");
                return Err(());
            }
        };

        let exa = PackedExa::new(name, instr);

        self.backbone.incoming().push(exa, t);

        Ok(())
    }

    fn parse_line(&mut self) -> Result<Command, Error> {
        match self.read_line() {
            Ok(_) => {}
            Err(_) => return Err(Error::ReadlineFail),
        }
        let args = match shlex::split(&self.readline_buffer) {
            Some(args) => args,
            None => return Err(Error::ParseFail),
        };
        let command = match Interactive::try_parse_from(args) {
            Ok(p) => p.command,
            Err(e) => {
                println!("{}", e);
                return Err(Error::ParseFail);
            }
        };
        Ok(command)
    }

    fn read_line(&mut self) -> Result<(), std::io::Error> {
        self.readline_buffer.clear();
        let mut stdin = std::io::stdin().lock();
        print!("{}> ", self.backbone.hostname());
        std::io::stdout().flush()?;
        stdin.read_line(&mut self.readline_buffer)?;
        Ok(())
    }
}
