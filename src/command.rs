use crate::{
    error::RunelParseError::{self, *},
    mode::RunelMode,
    widget::RunelWidget,
};
use std::{
    fmt::{self, Display},
    str::FromStr,
};

#[derive(Debug)]
pub enum RunelCommand {
    Set {
        widget: RunelWidget,
        value: String,
    },
    Mode {
        mode: RunelMode,
        command: Option<String>,
    },
    Reload,
    Quit,
}

impl RunelCommand {
    pub fn mode_command(&self) -> Option<&str> {
        match self {
            RunelCommand::Mode {
                command: Some(cmd), ..
            } => Some(cmd),
            _ => None,
        }
    }

    pub fn is_custom(&self) -> bool {
        match self {
            RunelCommand::Mode {
                mode: RunelMode::Custom,
                ..
            } => true,
            _ => false,
        }
    }

    fn parse_set(s: &str) -> Result<Self, RunelParseError> {
        s.find(':').map_or_else(
            || Err(InvalidSetSyntax),
            |n| match s.split_at(n) {
                (widget, value) => Ok(RunelCommand::Set {
                    widget: widget.parse()?,
                    value: value[1..].into(),
                }),
            },
        )
    }

    fn parse_mode(s: &str) -> Result<Self, RunelParseError> {
        Ok(RunelCommand::Mode {
            mode: s.parse()?,
            command: if s == "default" { None } else { Some(s.into()) },
        })
    }
}

impl FromStr for RunelCommand {
    type Err = RunelParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "reload" => Ok(RunelCommand::Reload),
            "quit" => Ok(RunelCommand::Quit),
            _ => s.find(':').map_or_else(
                || Err(InvalidSyntax),
                |n| match s.split_at(n) {
                    ("set", rest) => Self::parse_set(&rest[1..]),
                    ("mode", rest) => Self::parse_mode(&rest[1..]),
                    (cmd, _) => Err(InvalidCommand(cmd.into())),
                },
            ),
        }
    }
}

impl Display for RunelCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RunelCommand::Set { widget, value } => write!(f, "set:{}:{}", widget, value),
            RunelCommand::Mode { mode, .. } => write!(f, "mode:{}", mode),
            RunelCommand::Reload => write!(f, "reload"),
            RunelCommand::Quit => write!(f, "quit"),
        }
    }
}

#[cfg(test)]
mod command_tests {
    use super::*;

    #[test]
    fn command_reload_correct() {
        let res = RunelCommand::from_str("command:reload").unwrap();
        if let RunelCommand::Reload = res {
        } else {
            panic!("Incorrect command");
        }
    }

    #[test]
    fn command_quit_correct() {
        let res = RunelCommand::from_str("command:quit").unwrap();
        if let RunelCommand::Quit = res {
        } else {
            panic!("Incorrect command");
        }
    }

    #[test]
    fn mode_correct() {
        let res = RunelCommand::from_str("mode:some_cool_mode").unwrap();
        if let RunelCommand::Mode {
            command: Some(a), ..
        } = res
        {
            assert_eq!(a, "some_cool_mode".to_string());
        }
    }

    #[test]
    fn set_correct() {
        let res = RunelCommand::from_str("set:timer:off").unwrap();
        if let RunelCommand::Set { widget, value } = res {
            assert_eq!(widget, RunelWidget::Timer);
            assert_eq!(value, "off".to_string());
        }
    }

    #[test]
    #[should_panic]
    fn invalid_set() {
        RunelCommand::from_str("set:invalid").unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_command() {
        RunelCommand::from_str("command:invalid").unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_syntax() {
        RunelCommand::from_str("setaoesnth").unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_cmd() {
        RunelCommand::from_str("nono:bananas").unwrap();
    }

}
