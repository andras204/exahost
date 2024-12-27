#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExaStatus {
    Block(Block),
    SideEffect(SideEffect),
    Error(Error),
}

impl ExaStatus {
    pub fn is_side_effect(&self) -> bool {
        match self {
            Self::SideEffect(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Block {
    Send,
    Recv,
    Jump,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideEffect {
    Repl(u8),
    Link(i16),
    Kill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Halted,
    OutOfInstructions,
    FileNotFound,
    NoFileHeld,
    AlreadyHoldingFile,
    InvalidFileAccess,
    StorageFull,
    InvalidArgument,
    NumericValueRequired,
    InvalidHWRegisterAccess,
    UnknownInstruction,
}
