use std::io::BufRead;

use clap::Parser;
use command::Command;
use error::Error;
use parser::Interactive;

use crate::backbone::Backbone;

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
                Err(e) => {
                    println!("Error: {}", e);
                    continue;
                }
            };
            self.exec_command(command);
        }
    }

    fn exec_command(&self, command: Command) {}

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
            Err(_) => return Err(Error::ParseFail),
        };
        Ok(command)
    }

    fn read_line(&mut self) -> Result<(), std::io::Error> {
        self.readline_buffer.clear();
        let mut stdin = std::io::stdin().lock();
        stdin.read_line(&mut self.readline_buffer)?;
        Ok(())
    }
}
