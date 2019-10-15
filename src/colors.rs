use crate::error::XDisplayError;

use xrdb::{query_xrdb, XDisplay};

use std::{
    fmt::{self, Display},
    result::Result::Ok,
};

#[derive(Debug)]
enum ColorPlace {
    Background,
    Foreground,
}

impl Display for ColorPlace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ColorPlace::Background => write!(f, "B"),
            ColorPlace::Foreground => write!(f, "F"),
        }
    }
}

#[derive(Debug)]
pub struct Color {
    pub color: Option<String>,
    place: ColorPlace,
    xres: &'static str,
}

impl Color {
    pub fn new_background(color: Option<String>, xres: &'static str) -> Self {
        Color {
            color,
            place: ColorPlace::Background,
            xres,
        }
    }

    pub fn new_foreground(color: Option<String>, xres: &'static str) -> Self {
        Color {
            color,
            place: ColorPlace::Foreground,
            xres,
        }
    }

    fn query_xrdb(&mut self, display: &XDisplay) {
        self.color = query_xrdb(**display, self.xres, self.xres).map(|e| e.into());
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(c) = &self.color {
            write!(f, "%{{{}{}}}", self.place, c)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct Colors {
    pub bg: Color,
    pub free: Color,
    pub monitor: Color,
    pub occupied: Color,
    pub urgent: Color,
    pub state: Color,
    pub title: Color,
    pub info: Color,
    pub info_head: Color,
}

impl Colors {
    pub fn new() -> Self {
        Colors {
            bg: Color::new_background(None, "runel.bg"),
            free: Color::new_foreground(None, "runel.free"),
            monitor: Color::new_foreground(None, "runel.monitor"),
            occupied: Color::new_foreground(None, "runel.occupied"),
            urgent: Color::new_foreground(None, "runel.urgent"),
            state: Color::new_foreground(None, "runel.state"),
            title: Color::new_foreground(None, "runel.title"),
            info: Color::new_foreground(None, "runel.info"),
            info_head: Color::new_foreground(None, "runel.info_head"), // Head
        }
    }

    pub fn new_from_xresources() -> Result<Self, XDisplayError> {
        let mut colors = Colors::new();
        colors.query_xresources()?;
        Ok(colors)
    }

    pub fn query_xresources(&mut self) -> Result<(), XDisplayError> {
        let display = XDisplay::new().map_err(|_| XDisplayError)?;
        self.bg.query_xrdb(&display);
        self.free.query_xrdb(&display);
        self.monitor.query_xrdb(&display);
        self.occupied.query_xrdb(&display);
        self.urgent.query_xrdb(&display);
        self.state.query_xrdb(&display);
        self.title.query_xrdb(&display);
        self.info.query_xrdb(&display);
        self.info_head.query_xrdb(&display);
        Ok(())
    }
}
