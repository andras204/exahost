mod packed_exa;
mod register;
mod status;

pub use packed_exa::PackedExa;
pub use register::Register;
pub use status::{Block, Error, ExaStatus, SideEffect};

use crate::instruction::{Arg, Comp, Instruction, OpCode, RegLabel};
use crate::runtime::fs::File;
use crate::runtime::RuntimeHarness;

#[derive(Clone)]
pub struct Exa {
    pub name: String,
    pub instr_list: Box<[Instruction]>,
    pub instr_ptr: u8,
    pub repl_counter: usize,
    pub reg_x: Register,
    pub reg_t: Register,
    pub reg_f: Option<(i16, File)>,
    pub harness: RuntimeHarness,
}

impl Exa {
    // pub fn new(name: &str, instr_list: Box<[Instruction]>) -> Self {
    //     Self {
    //         name: name.to_string(),
    //         instr_list,
    //         instr_ptr: 0,
    //         repl_counter: 0,
    //         reg_x: Register::Number(0),
    //         reg_t: Register::Number(0),
    //         reg_f: None,
    //         harness: RuntimeHarness::default().into(),
    //     }
    // }

    pub fn exec(&mut self) -> Option<ExaStatus> {
        let instr = self.instr_list[self.instr_ptr as usize].clone();
        self.instr_ptr += 1;
        let status = match instr.0 {
            OpCode::Copy => self.copy(instr.two_args()),
            _ => Err(ExaStatus::Error(status::Error::Halted)),
        };

        match status {
            Ok(_) => {
                self.instr_ptr += 1;
                None
            }
            Err(res) => Some(res),
        }
    }

    fn copy(&mut self, (value, target): (Arg, Arg)) -> Result<(), ExaStatus> {
        let val = self.get_value(value)?;
        self.set_value(target.reg_label().unwrap(), val)?;
        Ok(())
    }

    fn jump(&mut self, target: Arg) -> Result<(), ExaStatus> {
        self.instr_ptr = target.jump_index().unwrap();
        Err(ExaStatus::Block(Block::Jump))
    }

    fn tjmp(&mut self, target: Arg) -> Result<(), ExaStatus> {
        if self.reg_t != Register::Number(0) && self.reg_t != Register::Keyword("".into()) {
            self.jump(target)
        } else {
            Ok(())
        }
    }

    fn fjmp(&mut self, target: Arg) -> Result<(), ExaStatus> {
        if self.reg_t == Register::Number(0) || self.reg_t == Register::Keyword("".into()) {
            self.jump(target)
        } else {
            Ok(())
        }
    }

    pub fn pack(self) -> PackedExa {
        PackedExa {
            name: self.name,
            instr_list: self.instr_list,
            instr_ptr: self.instr_ptr,
            repl_counter: self.repl_counter,
            reg_x: self.reg_x,
            reg_t: self.reg_t,
            reg_f: self.reg_f,
        }
    }

    fn get_value(&mut self, target: Arg) -> Result<Register, ExaStatus> {
        match target {
            Arg::Number(n) => Ok(Register::Number(n)),
            Arg::Keyword(k) => Ok(Register::Keyword(k)),
            Arg::RegLabel(rl) => match rl {
                RegLabel::X => Ok(self.reg_x.clone()),
                RegLabel::T => Ok(self.reg_t.clone()),
                _ => Err(ExaStatus::Error(status::Error::Halted)),
            },
            _ => Err(ExaStatus::Error(status::Error::InvalidArgument)),
        }
    }

    fn get_number(&mut self, target: Arg) -> Result<i16, ExaStatus> {
        match self.get_value(target)? {
            Register::Number(n) => Ok(n),
            Register::Keyword(_) => Err(ExaStatus::Error(status::Error::NumericValueRequired)),
        }
    }

    fn set_value(&mut self, target: RegLabel, value: Register) -> Result<(), ExaStatus> {
        match target {
            RegLabel::X => {
                self.reg_x = value;
                Ok(())
            }
            RegLabel::T => {
                self.reg_t = value;
                Ok(())
            }
            _ => Err(ExaStatus::Error(status::Error::Halted)),
        }
    }
}
