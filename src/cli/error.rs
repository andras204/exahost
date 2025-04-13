use std::fmt::Display;

pub enum Error {
    ReadlineFail,
    ParseFail,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadlineFail => write!(f, "failed to read line from stdin"),
            Self::ParseFail => write!(f, "failed to parse command"),
        }
    }
}
