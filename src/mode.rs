use crate::error::RunelParseError;
use std::{
    fmt::{self, Display},
    str::FromStr,
};

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum RunelMode {
    Default,
    Custom,
}

impl FromStr for RunelMode {
    type Err = RunelParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default" => Ok(RunelMode::Default),
            _ => Ok(RunelMode::Custom),
        }
    }
}

impl Display for RunelMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RunelMode::Default => write!(f, "default"),
            RunelMode::Custom => write!(f, "custom"),
        }
    }
}
