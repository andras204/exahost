pub enum ExecResult {
    SideEffect(SideEffect),
    Error(Error),
}

pub enum ExecStatus {
    Block,
    SideEffect(SideEffect),
    Error(Error),
}

pub enum SideEffect {
    Repl,
    Link,
    Kill,
}

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
