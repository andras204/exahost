use std::net::ToSocketAddrs;

use compiler::Compiler;
use exa::Exa;
use exavm::ExaVM;
use serde::{Deserialize, Serialize};

use crate::compiler::CompilerConfig;

pub mod compiler;
pub mod exa;
pub mod exavm;
pub mod linker;

pub struct Host {
    host_name: String,
    exa_vm: ExaVM,
    exa_compiler: Compiler,
}

impl Host {
    pub fn new(host_name: &str, bind_addr: &str) -> Host {
        println!("Initializing host: {}", host_name);
        let exa_compiler = Compiler::with_config(CompilerConfig::extended());
        Host {
            host_name: host_name.to_string(),
            exa_vm: ExaVM::new(),
            exa_compiler,
        }
    }

    pub fn compile_exa(&self, name: &str, instructions: Vec<String>) -> Result<Exa, Vec<String>> {
        let instr = self.exa_compiler.compile(instructions)?;
        Ok(Exa::new(name, instr))
    }

    pub fn add_exa(&mut self, exa: Exa) {
        self.exa_vm.add_exa(exa);
    }

    pub fn step(&mut self) {
        self.exa_vm.step();
    }

    pub fn connect(&mut self, address: &(impl ToSocketAddrs + ?Sized)) {
        unimplemented!()
    }

    fn load_config() -> HostConfiguration {
        unimplemented!()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HostConfiguration {
    compiler_configuration: CompilerConfig,
}
