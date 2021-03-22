use {
    crate::config::FIELDS,
    std::{
        fmt::{self, Display},
        str::FromStr,
    },
};

#[derive(Clone)]
pub struct Field(String);

impl FromStr for Field {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if FIELDS.iter().any(|f| f.0 == s) {
            Ok(Self(s.into()))
        } else {
            Err(format!(
                "Field {} is not valid;\nallowed values: {:?}",
                s,
                FIELDS.iter().map(|f| f.0).collect::<Vec<_>>()
            ))
        }
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl AsRef<str> for Field {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
