use dirs;
use std::{
    fs::DirBuilder,
    io::Result as IoResult,
    path::{Path, PathBuf},
};

const DIR: &str = "runel";

fn create<P: AsRef<Path>>(path: P) -> IoResult<()> {
    DirBuilder::new().recursive(true).create(path)
}

fn cache_path(path: &str) -> IoResult<PathBuf> {
    let mut dir = dirs::cache_dir().unwrap();
    dir.push(DIR);
    create(&dir)?;
    dir.push(path);
    Ok(dir)
}

pub fn daemon_err() -> IoResult<PathBuf> {
    cache_path("daemon.err")
}

pub fn daemon_out() -> IoResult<PathBuf> {
    cache_path("daemon.out")
}

pub fn mode_err() -> IoResult<PathBuf> {
    cache_path("mode.err")
}

pub fn mode_out() -> IoResult<PathBuf> {
    cache_path("mode.out")
}

fn runtime_path(path: &str) -> IoResult<PathBuf> {
    dirs::runtime_dir().map_or_else(
        || Ok(PathBuf::from(String::from("/tmp/runel.") + path)),
        |mut dir| {
            dir.push(DIR);
            create(&dir)?;
            dir.push(path);
            Ok(dir)
        },
    )
}

pub fn socket() -> IoResult<PathBuf> {
    runtime_path("socket")
}

pub fn pid() -> IoResult<PathBuf> {
    runtime_path("pid")
}

pub fn mode<P: AsRef<Path>>(path: P) -> IoResult<PathBuf> {
    let mut dir = dirs::config_dir().unwrap();
    dir.push(DIR);
    create(&dir)?;
    dir.push(path);
    Ok(dir)
}
