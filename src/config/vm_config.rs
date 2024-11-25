use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VMConfig {
    pub max_exas: usize,
    pub max_files: usize,
}

impl VMConfig {
    pub fn new(max_exas: usize, max_files: usize) -> Self {
        Self {
            max_exas,
            max_files,
        }
    }
}

impl Default for VMConfig {
    fn default() -> Self {
        Self::new(9, 9)
    }
}
