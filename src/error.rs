use std::{
    convert::From,
    error::Error,
    ffi,
    fmt::{self, Display},
    io,
    num::ParseFloatError,
};

#[derive(Debug)]
pub enum PrinterError {
    IoError(io::Error),
    FloatError(ParseFloatError),
    RegexError(regex::Error),
    BspwmNotFound,
    SelemNotFound,
    AlsaError(alsa::Error),
    FfiNulError(ffi::NulError),
    RunelParseError(RunelParseError),
    Unreachable(String),
    NixError(nix::Error),
    KbdError(&'static str),
}

impl Error for PrinterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use PrinterError::*;

        match self {
            IoError(e) => Some(e),
            FloatError(e) => Some(e),
            RegexError(e) => Some(e),
            BspwmNotFound => None,
            SelemNotFound => None,
            AlsaError(e) => Some(e),
            FfiNulError(e) => Some(e),
            RunelParseError(e) => Some(e),
            Unreachable(_) => None,
            NixError(e) => Some(e),
            KbdError(_) => None,
        }
    }
}

impl Display for PrinterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use PrinterError::*;

        match self {
            IoError(e) => write!(f, "{}", e),
            FloatError(e) => write!(f, "{}", e),
            RegexError(e) => write!(f, "{}", e),
            BspwmNotFound => write!(f, "Can't find bspwm socket"),
            SelemNotFound => write!(f, "Can't find alsa selem"),
            AlsaError(e) => write!(f, "{}", e),
            FfiNulError(e) => write!(f, "{}", e),
            RunelParseError(e) => write!(f, "{}", e),
            Unreachable(e) => write!(f, "Reached unreachable in main loop of printer '{}'", e),
            NixError(e) => write!(f, "{}", e),
            KbdError(e) => write!(f, "{}", e),
        }
    }
}

impl From<nix::Error> for PrinterError {
    fn from(source: nix::Error) -> Self {
        PrinterError::NixError(source)
    }
}

impl From<RunelParseError> for PrinterError {
    fn from(source: RunelParseError) -> Self {
        PrinterError::RunelParseError(source)
    }
}

impl From<ffi::NulError> for PrinterError {
    fn from(source: ffi::NulError) -> Self {
        PrinterError::FfiNulError(source)
    }
}

impl From<alsa::Error> for PrinterError {
    fn from(source: alsa::Error) -> Self {
        PrinterError::AlsaError(source)
    }
}

impl From<regex::Error> for PrinterError {
    fn from(source: regex::Error) -> Self {
        PrinterError::RegexError(source)
    }
}

impl From<ParseFloatError> for PrinterError {
    fn from(source: ParseFloatError) -> Self {
        PrinterError::FloatError(source)
    }
}

impl From<io::Error> for PrinterError {
    fn from(source: io::Error) -> Self {
        PrinterError::IoError(source)
    }
}

#[derive(Debug)]
pub enum RunelParseError {
    InvalidSyntax,
    InvalidCommand(String),
    InvalidSetSyntax,
    InvalidWidget(String),
}

impl Error for RunelParseError {}

impl Display for RunelParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use RunelParseError::*;

        match self {
            InvalidSyntax => write!(
                f,
                "Could not find ':' char. The command \
                 syntax is wrong"
            ),
            InvalidCommand(s) => write!(f, "'{}' is not a valid command", s),
            InvalidSetSyntax => write!(
                f,
                "Syntax for 'set' command is not valid: could not \
                 find ':' char or the value is empty"
            ),
            InvalidWidget(s) => write!(f, "Non existent widget '{}'", s),
        }
    }
}

#[derive(Debug)]
pub struct XDisplayError;

impl Error for XDisplayError {}

impl Display for XDisplayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not get XDisplay")
    }
}
