#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExaStatus {
    Block(Block),
    SideEffect(SideEffect),
    Error(Error),
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
    InvalidArgument,
    NumericValueRequired,
}
