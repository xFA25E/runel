use {
    crate::config::CONFIG_DIR,
    std::{
        fmt::{self, Display},
        fs::DirBuilder,
        path::PathBuf,
        str::FromStr,
    },
};

#[derive(Debug, PartialEq, Eq)]
pub struct Mode {
    pub mode: String,
    pub path: PathBuf,
}

impl FromStr for Mode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut path = dirs::config_dir().unwrap();
        path.push(CONFIG_DIR);
        DirBuilder::new()
            .recursive(true)
            .create(&path)
            .map_err(|e| format!("{}", e))?;
        path.push(s);

        if !(path.is_file() && path.exists()) {
            Err(format!("Mode \"{}\" does not exists", s))
        } else {
            Ok(Self {
                mode: s.into(),
                path,
            })
        }
    }
}

impl Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.mode)
    }
}
