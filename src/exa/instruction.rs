use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

use crate::exa::Arg;

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Encode, Decode)]
pub enum OpCode {
    /// `COPY value: R/N target: R`
    ///
    /// copies `value` into `target`
    Copy,
    /// `VOID target: R`
    ///
    /// clears target by:
    /// - setting `X` and `T` to default values (0)
    /// - taking and discarding the value in `M`
    /// - blanking the value in `F`
    Void,

    /// `ADDI num1: R/N num2: R/N target: R`
    ///
    /// performs `num1 + num2`, and puts the result in `target`
    ///
    /// Errors:
    /// - `num1` or `num2` are not numeric values
    Addi,
    /// `SUBI num1: R/N num2: R/N target: R`
    ///
    /// performs `num1 - num2`, and puts the result in `target`
    ///
    /// Errors:
    /// - `num1` or `num2` are not numeric values
    Subi,
    /// `MULI num1: R/N num2: R/N target: R`
    ///
    /// performs `num1 * num2`, and puts the result in `target`
    ///
    /// Errors:
    /// - `num1` or `num2` are not numeric values
    Muli,
    /// `DIVI num1: R/N num2: R/N target: R`
    ///
    /// performs `num1 / num2`, and puts the result in `target`
    ///
    /// Errors:
    /// - `num1` or `num2` are not numeric values
    Divi,
    /// `MODI num1: R/N num2: R/N target: R`
    ///
    /// performs `num1 % num2`, and puts the result in `target`
    ///
    /// Errors:
    /// - `num1` or `num2` are not numeric values
    Modi,
    /// `INST ...`
    ///
    /// TODO: docs
    Swiz,

    /// doesn't work
    Mode,

    /// `TEST val1: R/N comp: C val2: R/N`
    ///
    /// performs `val1 comp val2`, and puts the result in `T`
    ///
    /// basic comparison operators:
    /// - `>` (Gt)
    /// - `<` (Lt)
    /// - `=` (Eq)
    ///
    /// extended comparison operators:
    /// - `>` (Gt)
    /// - `<` (Lt)
    /// - `=` (Eq)
    /// - `>=` (Ge)
    /// - `<=` (Le)
    /// - `!=` (Ne)
    Test,
    /// `TEST MRD`
    ///
    /// checks if `M` can be read without blocking,
    /// and puts result into `T`
    TestMrd,
    /// `TEST EOF`
    ///
    /// checks if `F` is at the end of the held file,
    /// and puts result into `T`
    ///
    /// Errors:
    /// - file not held
    TestEof,

    /// `MARK label: L`
    ///
    /// Pseudo-instruction, target of `JUMP` and `REPL` instructions
    Mark,
    /// `JUMP label: L`
    ///
    /// jumps to `label`
    Jump,
    /// `FJMP label: L`
    ///
    /// jumps to `label` if `T` is 0
    Fjmp,
    /// `TJMP label: L`
    ///
    /// jumps to `label` if `T` is not 0
    Tjmp,

    /// `MAKE`
    ///
    /// creates a file
    ///
    /// Errors:
    /// - already holding file
    Make,
    /// `GRAB id: R/N`
    ///
    /// grabs file with `id`
    ///
    /// Errors:
    /// - already holding file
    Grab,

    /// `FILE target: R`
    ///
    /// puts if of held file into `target`
    ///
    /// Errors:
    /// - file not held
    File,
    /// `SEEK amount: R/N`
    ///
    /// moves `F` forward in held file by `amount` (or backwards if `amount` is negative)
    ///
    /// Errors:
    /// - file not held
    Seek,
    /// `DROP`
    ///
    /// drops held file
    ///
    /// Blocks until there is space for the file
    ///
    /// Errors:
    /// - file not held
    Drop,
    /// `WIPE`
    ///
    /// deletes held file
    ///
    /// Errors:
    /// - file not held
    Wipe,

    /// `LINK id: R/N`
    ///
    /// UNIMPLEMENTED!
    Link,
    /// `REPL label: L`
    ///
    /// spawns a copy of the EXA, starting execution at `label`
    ///
    /// Blocks until there is space for the copy
    Repl,
    /// `HALT`
    ///
    /// destroys EXA
    Halt,
    /// `HALT`
    ///
    /// destroys another EXA in the current host
    Kill,

    /// `INST ...`
    ///
    /// TODO: docs
    Rand,
    /// `HOST target: R`
    ///
    /// puts the name of the current host into `target`
    Host,

    /// `NOOP`
    ///
    /// does nothing for 1 cycle
    Noop,
    /// `INST ...`
    ///
    /// TODO: docs
    Prnt,
}

impl FromStr for OpCode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "copy" => Ok(Self::Copy),
            "void" => Ok(Self::Void),
            "addi" => Ok(Self::Addi),
            "subi" => Ok(Self::Subi),
            "muli" => Ok(Self::Muli),
            "divi" => Ok(Self::Divi),
            "modi" => Ok(Self::Modi),
            "swiz" => Ok(Self::Swiz),
            "mode" => Ok(Self::Mode),
            "test" => Ok(Self::Test),
            "test mrd" => Ok(Self::TestMrd),
            "test eof" => Ok(Self::TestEof),
            "mark" => Ok(Self::Mark),
            "jump" => Ok(Self::Jump),
            "fjmp" => Ok(Self::Fjmp),
            "tjmp" => Ok(Self::Tjmp),
            "grab" => Ok(Self::Grab),
            "file" => Ok(Self::File),
            "seek" => Ok(Self::Seek),
            "drop" => Ok(Self::Drop),
            "wipe" => Ok(Self::Wipe),
            "link" => Ok(Self::Link),
            "repl" => Ok(Self::Repl),
            "halt" => Ok(Self::Halt),
            "kill" => Ok(Self::Kill),
            "rand" => Ok(Self::Rand),
            "host" => Ok(Self::Host),
            "noop" => Ok(Self::Noop),
            "prnt" => Ok(Self::Prnt),
            _ => Err(format!("could not parse '{}' as OpCode", s)),
        }
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Copy => write!(f, "COPY"),
            Self::Void => write!(f, "VOID"),
            Self::Addi => write!(f, "ADDI"),
            Self::Subi => write!(f, "SUBI"),
            Self::Muli => write!(f, "MULI"),
            Self::Divi => write!(f, "DIVI"),
            Self::Modi => write!(f, "MODI"),
            Self::Swiz => write!(f, "SWIZ"),
            Self::Mode => write!(f, "MODE"),
            Self::Test => write!(f, "TEST"),
            Self::TestMrd => write!(f, "TEST MRD"),
            Self::TestEof => write!(f, "TEST EOF"),
            Self::Mark => write!(f, "MARK"),
            Self::Jump => write!(f, "JUMP"),
            Self::Fjmp => write!(f, "FJMP"),
            Self::Tjmp => write!(f, "TJMP"),
            Self::Make => write!(f, "MAKE"),
            Self::Grab => write!(f, "GRAB"),
            Self::File => write!(f, "FILE"),
            Self::Seek => write!(f, "SEEK"),
            Self::Drop => write!(f, "DROP"),
            Self::Wipe => write!(f, "WIPE"),
            Self::Link => write!(f, "LINK"),
            Self::Repl => write!(f, "REPL"),
            Self::Halt => write!(f, "HALT"),
            Self::Kill => write!(f, "KILL"),
            Self::Rand => write!(f, "RAND"),
            Self::Host => write!(f, "HOST"),
            Self::Noop => write!(f, "NOOP"),
            Self::Prnt => write!(f, "PRNT"),
        }
    }
}
