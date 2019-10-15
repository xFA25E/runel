use crate::error::RunelParseError;
use std::{
    fmt::{self, Display},
    str::FromStr,
};

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum RunelWidget {
    Space,
    Left,
    Right,
    Custom,
    Bspwm,
    Title,
    Timer,
    Keyboard,
    Brightness,
    Volume,
    Battery,
    Date,
    Clock,
}

impl Display for RunelWidget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RunelWidget::Space => "space".fmt(f),
            RunelWidget::Left => "left".fmt(f),
            RunelWidget::Right => "right".fmt(f),
            RunelWidget::Custom => "custom".fmt(f),
            RunelWidget::Bspwm => "bspwm".fmt(f),
            RunelWidget::Title => "title".fmt(f),
            RunelWidget::Timer => "timer".fmt(f),
            RunelWidget::Keyboard => "keyboard".fmt(f),
            RunelWidget::Brightness => "brightness".fmt(f),
            RunelWidget::Volume => "volume".fmt(f),
            RunelWidget::Battery => "battery".fmt(f),
            RunelWidget::Date => "date".fmt(f),
            RunelWidget::Clock => "clock".fmt(f),
        }
    }
}

impl FromStr for RunelWidget {
    type Err = RunelParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "space" => Ok(RunelWidget::Space),
            "left" => Ok(RunelWidget::Left),
            "right" => Ok(RunelWidget::Right),
            "custom" => Ok(RunelWidget::Custom),
            "bspwm" => Ok(RunelWidget::Bspwm),
            "title" => Ok(RunelWidget::Title),
            "timer" => Ok(RunelWidget::Timer),
            "keyboard" => Ok(RunelWidget::Keyboard),
            "brightness" => Ok(RunelWidget::Brightness),
            "volume" => Ok(RunelWidget::Volume),
            "battery" => Ok(RunelWidget::Battery),
            "date" => Ok(RunelWidget::Date),
            "clock" => Ok(RunelWidget::Clock),
            _ => Err(RunelParseError::InvalidWidget(s.into())),
        }
    }
}
