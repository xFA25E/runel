use std::{
    fmt::{self, Display},
    str::FromStr,
};

pub struct Color(Option<String>);

impl FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(Self(None))
        } else if s.len() == 7
            && s.starts_with("#")
            && s.chars().skip(1).all(|c| c.is_ascii_hexdigit())
        {
            Ok(Self(Some(s.into())))
        } else {
            Err(format!("Invalid hex color: {}", s))
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(c) = &self.0 {
            write!(f, "%{{F{}}}", c)
        } else {
            Ok(())
        }
    }
}

impl AsRef<Option<String>> for Color {
    fn as_ref(&self) -> &Option<String> {
        &self.0
    }
}
