mod packed_exa;
mod register;
pub mod status;

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

    pub fn exec(&mut self) -> Option<ExaStatus> {
        if self.instr_ptr as usize == self.instr_list.len() {
            return Some(ExaStatus::Error(Error::OutOfInstructions));
        }

        let instr = self.instr_list[self.instr_ptr as usize].clone();

        let status = match instr.0 {
            OpCode::Copy => self.copy(instr.two_args()),
            OpCode::Void => self.void(instr.one_arg()),
            OpCode::Addi => self.addi(instr.three_args()),
            OpCode::Subi => self.subi(instr.three_args()),
            OpCode::Muli => self.muli(instr.three_args()),
            OpCode::Divi => self.divi(instr.three_args()),
            OpCode::Modi => self.modi(instr.three_args()),
            OpCode::Swiz => self.swiz(instr.three_args()),
            OpCode::Test => self.test(instr.three_args()),
            OpCode::TestMrd => self.test_mrd(),
            OpCode::TestEof => self.test_eof(),
            OpCode::Mark => unreachable!(),
            OpCode::Jump => self.jump(instr.one_arg()),
            OpCode::Fjmp => self.fjmp(instr.one_arg()),
            OpCode::Tjmp => self.tjmp(instr.one_arg()),
            OpCode::Make => self.make(),
            OpCode::Grab => self.grab(instr.one_arg()),
            OpCode::File => self.file(instr.one_arg()),
            OpCode::Seek => self.seek(instr.one_arg()),
            OpCode::Drop => self.drop(),
            OpCode::Wipe => self.wipe(),
            OpCode::Link => self.link(instr.one_arg()),
            OpCode::Repl => self.repl(instr.one_arg()),
            OpCode::Halt => Self::halt(),
            OpCode::Kill => Self::kill(),
            OpCode::Rand => self.rand(instr.three_args()),
            OpCode::Host => self.host(instr.one_arg()),
            OpCode::Noop => Ok(()),
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
        self.set_value(target.reg_label().unwrap(), val)
    }

    fn void(&mut self, target: Arg) -> Result<(), ExaStatus> {
        match target.reg_label()? {
            RegLabel::X => {
                self.reg_x = Register::zero();
                Ok(())
            }
            RegLabel::T => {
                self.reg_t = Register::zero();
                Ok(())
            }
            RegLabel::F => self.set_value(RegLabel::F, Register::empty()),
            RegLabel::M => self.get_value(Arg::RegLabel(RegLabel::M)).map(|_| ()),
            RegLabel::H(_) => Err(ExaStatus::Error(Error::InvalidHWRegisterAccess)),
        }
    }

    fn addi(&mut self, (a, b, target): (Arg, Arg, Arg)) -> Result<(), ExaStatus> {
        let a = self.get_number(a)?;
        let b = self.get_number(b)?;
        self.set_value(target.reg_label()?, Register::Number(a + b))
    }

    fn subi(&mut self, (a, b, target): (Arg, Arg, Arg)) -> Result<(), ExaStatus> {
        let a = self.get_number(a)?;
        let b = self.get_number(b)?;
        self.set_value(target.reg_label()?, Register::Number(a - b))
    }

    fn muli(&mut self, (a, b, target): (Arg, Arg, Arg)) -> Result<(), ExaStatus> {
        let a = self.get_number(a)?;
        let b = self.get_number(b)?;
        self.set_value(target.reg_label()?, Register::Number(a * b))
    }

    fn divi(&mut self, (a, b, target): (Arg, Arg, Arg)) -> Result<(), ExaStatus> {
        let a = self.get_number(a)?;
        let b = self.get_number(b)?;
        self.set_value(target.reg_label()?, Register::Number(a / b))
    }

    fn modi(&mut self, (a, b, target): (Arg, Arg, Arg)) -> Result<(), ExaStatus> {
        let a = self.get_number(a)?;
        let b = self.get_number(b)?;
        self.set_value(target.reg_label()?, Register::Number(a % b))
    }

    fn swiz(&mut self, (a, b, target): (Arg, Arg, Arg)) -> Result<(), ExaStatus> {
        let a = self.get_number(a)?;
        let b = self.get_number(b)?;
        let mut result = 0;
        for x in 1..5 {
            let mask = match (b.abs() % 10i16.pow(x)) / 10i16.pow(x - 1) {
                1 => 1,
                2 => 2,
                3 => 3,
                4 => 4,
                _ => continue,
            };
            result += ((a.abs() % 10i16.pow(mask)) / 10i16.pow(mask - 1)) * 10i16.pow(x - 1);
        }
        result *= a.signum() * b.signum();
        self.set_value(target.reg_label()?, Register::Number(result))
    }

    fn rand(&mut self, (a, b, target): (Arg, Arg, Arg)) -> Result<(), ExaStatus> {
        let a = self.get_number(a)?;
        let b = self.get_number(b)?;
        self.set_value(target.reg_label()?, self.harness.rand(a, b))
    }

    fn test(&mut self, (a, comp, b): (Arg, Arg, Arg)) -> Result<(), ExaStatus> {
        let a = self.get_value(a)?;
        let b = self.get_value(b)?;
        let t = match comp.comp()? {
            Comp::Eq => a == b,
            Comp::Gt => a > b,
            Comp::Lt => a < b,
            Comp::Ge => a >= b,
            Comp::Le => a <= b,
            Comp::Ne => a != b,
        };
        self.set_value(RegLabel::T, Register::Number(t as i16))
    }

    fn test_eof(&mut self) -> Result<(), ExaStatus> {
        let t = match &self.reg_f {
            Some((_, f)) => f.is_eof(),
            None => return Err(ExaStatus::Error(status::Error::NoFileHeld)),
        };
        self.set_value(RegLabel::T, Register::Number(t as i16))
    }

    fn test_mrd(&mut self) -> Result<(), ExaStatus> {
        self.set_value(
            RegLabel::T,
            Register::Number(self.harness.is_m_read_non_block() as i16),
        )
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

    fn make(&mut self) -> Result<(), ExaStatus> {
        match self.harness.make_file() {
            Some(fh) => {
                self.reg_f = Some(fh);
                Ok(())
            }
            None => Err(ExaStatus::Error(Error::StorageFull)),
        }
    }

    fn grab(&mut self, target: Arg) -> Result<(), ExaStatus> {
        let id = self.get_number(target)?;
        match self.harness.grab_file(id) {
            Some(fh) => {
                self.reg_f = Some(fh);
                Ok(())
            }
            None => Err(ExaStatus::Error(Error::FileNotFound)),
        }
    }

    fn file(&mut self, target: Arg) -> Result<(), ExaStatus> {
        if self.reg_f.is_some() {
            self.set_value(
                target.reg_label()?,
                Register::Number(self.reg_f.as_ref().unwrap().0),
            )?;
            Ok(())
        } else {
            Err(ExaStatus::Error(Error::NoFileHeld))
        }
    }

    fn seek(&mut self, offset: Arg) -> Result<(), ExaStatus> {
        if self.reg_f.is_some() {
            let offset = self.get_number(offset)?;
            self.reg_f.as_mut().unwrap().1.seek(offset);
            Ok(())
        } else {
            Err(ExaStatus::Error(Error::NoFileHeld))
        }
    }

    fn drop(&mut self) -> Result<(), ExaStatus> {
        if self.reg_f.is_some() {
            self.harness.return_file(self.reg_f.take().unwrap());
            Ok(())
        } else {
            Err(ExaStatus::Error(Error::NoFileHeld))
        }
    }

    fn wipe(&mut self) -> Result<(), ExaStatus> {
        if self.reg_f.is_some() {
            self.harness.wipe_file(self.reg_f.take().unwrap().0);
            Ok(())
        } else {
            Err(ExaStatus::Error(Error::NoFileHeld))
        }
    }

    fn link(&mut self, link: Arg) -> Result<(), ExaStatus> {
        Err(ExaStatus::SideEffect(SideEffect::Link(
            self.get_number(link)?,
        )))
    }

    fn repl(&mut self, label: Arg) -> Result<(), ExaStatus> {
        Err(ExaStatus::SideEffect(SideEffect::Repl(label.jump_index()?)))
    }

    fn halt() -> Result<(), ExaStatus> {
        Err(ExaStatus::Error(Error::Halted))
    }

    fn kill() -> Result<(), ExaStatus> {
        Err(ExaStatus::SideEffect(SideEffect::Kill))
    }

    fn host(&mut self, target: Arg) -> Result<(), ExaStatus> {
        self.set_value(target.reg_label()?, self.harness.hostname())
    }

    fn get_value(&mut self, target: Arg) -> Result<Register, ExaStatus> {
        match target {
            Arg::Number(n) => Ok(Register::Number(n)),
            Arg::Keyword(k) => Ok(Register::Keyword(k)),
            Arg::RegLabel(rl) => match rl {
                RegLabel::X => Ok(self.reg_x.clone()),
                RegLabel::T => Ok(self.reg_t.clone()),
                RegLabel::F => {
                    if self.reg_f.is_some() {
                        Ok(self
                            .reg_f
                            .as_mut()
                            .unwrap()
                            .1
                            .read()
                            .unwrap_or(Register::Keyword("".into())))
                    } else {
                        Err(ExaStatus::Error(Error::NoFileHeld))
                    }
                }
                RegLabel::M => self.harness.recv(),
                RegLabel::H(h) => self.harness.hw_read(&self, h),
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
            RegLabel::F => {
                if self.reg_f.is_some() {
                    self.reg_f.as_mut().unwrap().1.write(value);
                    Ok(())
                } else {
                    Err(ExaStatus::Error(Error::NoFileHeld))
                }
            }
            RegLabel::M => self.harness.send(value),
            RegLabel::H(h) => self.harness.hw_write(&self, h, value),
        }
    }
}
