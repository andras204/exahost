use crate::exa::{
    status::{self, ExaStatus},
    Exa, Register,
};

pub trait HardwareRegister: std::fmt::Debug {
    fn label_str(&self) -> Box<str>;

    fn read(&mut self, exa: &Exa) -> Result<Register, ExaStatus>;

    fn write(&mut self, exa: &Exa, value: Register) -> Result<(), ExaStatus>;
}

#[derive(Debug, Clone)]
pub struct PrintRegister;

impl HardwareRegister for PrintRegister {
    fn label_str(&self) -> Box<str> {
        "#prnt".to_uppercase().into()
    }

    fn read(&mut self, exa: &Exa) -> Result<Register, ExaStatus> {
        Err(ExaStatus::Error(status::Error::InvalidHWRegisterAccess))
    }

    fn write(&mut self, exa: &Exa, value: Register) -> Result<(), ExaStatus> {
        println!("{}> {}", exa.name, value);
        Ok(())
    }
}
