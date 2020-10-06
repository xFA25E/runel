use std::{
    fmt::{self, Display},
    str::FromStr,
};

pub struct Color(Option<String>);
pub struct DrawColor<'a, D: Display>(&'a Color, D);

impl Color {
    pub fn draw<D: Display>(&self, element: D) -> DrawColor<D> {
        DrawColor(self, element)
    }
}

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

impl<'a, D: Display> Display for DrawColor<'a, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(c) = &(self.0).0 {
            write!(f, "%{{F{}}}{}%{{F-}}", c, self.1)
        } else {
            write!(f, "{}", self.1)
        }
    }
}

impl AsRef<Option<String>> for Color {
    fn as_ref(&self) -> &Option<String> {
        &self.0
    }
}
