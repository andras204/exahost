use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

mod arg;
mod opcode;

pub use arg::{Arg, Comp, RegLabel};
pub use opcode::OpCode;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub struct Instruction(
    pub OpCode,
    pub Option<Arg>,
    pub Option<Arg>,
    pub Option<Arg>,
);

impl Instruction {
    pub fn destructure(self) -> (OpCode, Option<Arg>, Option<Arg>, Option<Arg>) {
        (self.0, self.1, self.2, self.3)
    }

    pub fn one_arg(self) -> Arg {
        self.1.unwrap()
    }

    pub fn two_args(self) -> (Arg, Arg) {
        (self.1.unwrap(), self.2.unwrap())
    }

    pub fn three_args(self) -> (Arg, Arg, Arg) {
        (self.1.unwrap(), self.2.unwrap(), self.3.unwrap())
    }

    pub fn args(self) -> (Option<Arg>, Option<Arg>, Option<Arg>) {
        (self.1, self.2, self.3)
    }

    pub fn arg_refs(&self) -> (Option<&Arg>, Option<&Arg>, Option<&Arg>) {
        (self.1.as_ref(), self.2.as_ref(), self.3.as_ref())
    }
}
