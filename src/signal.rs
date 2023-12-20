use crate::exa::Exa;

#[derive(Debug, Clone)]
pub enum ExaSignal {
    Ok,
    Err(String),
    Repl(Exa),
    Halt,
    Kill,
    Link(i16),
    Tx,
    Rx,
}

#[derive(Debug, Clone)]
pub enum HostSignal {
    Link((i16, Exa)),
    Step,
    Stop,
}
