use crate::cli::command::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VMCommand {
    Run,
    Step,
    Stop,
    KillAll,
}

impl TryFrom<Command> for VMCommand {
    type Error = ();
    fn try_from(value: Command) -> Result<Self, Self::Error> {
        match value {
            Command::Run => Ok(Self::Run),
            Command::Step => Ok(Self::Step),
            Command::Stop => Ok(Self::Stop),
            Command::KillAll => Ok(Self::KillAll),
            _ => Err(()),
        }
    }
}
