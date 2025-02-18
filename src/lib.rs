use std::{
    io::{Read, Write},
    net::ToSocketAddrs,
};

use compiler::Compiler;
use config::{CompilerConfig, HostConfig, VMConfig};
use exa::PackedExa;
use runtime::{Runtime, SharedRT};
use server::Server;
use vm::{bridge::VMBridge, VM};

pub mod compiler;
pub mod config;
pub mod exa;
pub mod instruction;
pub mod runtime;
pub mod server;
pub mod vm;

pub struct Host {
    compiler: Compiler,
    vm: VM,
    pub server: Server,
    runtime: SharedRT,
    config: HostConfig,
}

impl Host {
    pub fn new(host_name: &str, bind_addr: &str) -> Host {
        println!("Initializing host: {}", host_name);
        let exa_compiler = Compiler::new(CompilerConfig::extended());
        let vm_config: VMConfig = VMConfig::default();
        let hostname: Box<str> = host_name.into();
        let runtime = Runtime::new(&hostname).make_shared();
        let vm = VM::new(runtime.clone(), 32);
        let mut server = Server::new(vm.get_bridge());
        server.start_listening(bind_addr.to_owned());

        Host {
            compiler: exa_compiler,
            vm,
            server,
            runtime,
            config: HostConfig {
                hostname: hostname.into(),
                compiler_config: CompilerConfig::extended().into(),
                vm_config: vm_config.into(),
            },
        }
    }

    // pub fn init() -> Host {
    //     let config = Self::load_config();
    //     println!("Initializing host: {}", config.hostname);
    //     Host {
    //         compiler: Compiler::new((*config.compiler_config).clone()),
    //         vm: VM::new(config.hostname.clone(), config.vm_config.clone()),
    //         config,
    //     }
    // }

    pub fn compile_exa(
        &self,
        name: &str,
        instructions: Vec<&str>,
    ) -> Result<PackedExa, Vec<compiler::Error>> {
        let instr = self.compiler.compile(&instructions)?;
        Ok(PackedExa::new(name, instr))
    }

    pub fn add_exa(&mut self, exa: PackedExa) {
        self.vm.add_exa(exa);
    }

    // pub fn add_file(&mut self, file: File) {
    //     self.vm.add_file(file);
    // }

    pub fn step(&mut self) {
        self.vm.step();
    }

    pub fn connect(&mut self, address: &(impl ToSocketAddrs + ?Sized)) {
        self.server.connect(address);
    }

    pub fn save_config(&self) -> Result<(), std::io::Error> {
        let s = toml::to_string_pretty(&self.config).unwrap();
        std::fs::File::create("hosts/config.toml")?.write_all(s.as_bytes())?;
        Ok(())
    }

    fn load_config() -> HostConfig {
        let mut s = String::new();
        let res = std::fs::File::open("hosts/config.toml");
        match res {
            Ok(mut file) => match file.read_to_string(&mut s) {
                Ok(_) => match toml::from_str(&s) {
                    Ok(config) => config,
                    Err(_) => {
                        eprintln!("unable to parse configuration file, using defaults");
                        HostConfig::default()
                    }
                },
                Err(_) => {
                    eprintln!("unable to read configuration file, using defaults");
                    HostConfig::default()
                }
            },
            Err(_) => {
                eprintln!("unable to open configuration file, using defaults");
                HostConfig::default()
            }
        }
    }
}

impl Default for Host {
    fn default() -> Self {
        Self::new("Rhizome", "0.0.0.0:6800")
    }
}
