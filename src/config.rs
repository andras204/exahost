use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::compiler::Config as CompilerConfig;
use crate::vm::Config as VMConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostConfig {
    pub hostname: Rc<Box<str>>,
    pub compiler_config: Rc<CompilerConfig>,
    pub vm_config: Rc<VMConfig>,
}

impl HostConfig {
    pub fn new(
        hostname: Rc<Box<str>>,
        compiler_config: Rc<CompilerConfig>,
        vm_config: Rc<VMConfig>,
    ) -> Self {
        Self {
            hostname,
            compiler_config,
            vm_config,
        }
    }
}

impl Default for HostConfig {
    fn default() -> Self {
        Self::new(
            Rc::new("Rhizome".into()),
            CompilerConfig::default().into(),
            VMConfig::default().into(),
        )
    }
}
