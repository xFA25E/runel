use crate::panel::Panel;

use std::{
    fmt::Write as FmtWrite,
    io::{self, Write as IoWrite},
    process::{Child, ChildStdin, Command, Stdio},
};

pub struct Lemonbar {
    proc: Child,
    stdin: ChildStdin,
    buf: String,
}

impl Lemonbar {
    pub fn start(args: Vec<String>) -> Result<Self, io::Error> {
        let mut proc = Command::new("lemonbar")
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()?;
        let stdin = proc.stdin.take().unwrap();

        Ok(Self {
            proc,
            stdin,
            buf: String::new(),
        })
    }

    pub fn send(&mut self, panel: &Panel) -> Result<(), io::Error> {
        self.buf.clear();
        write!(self.buf, "{}", panel).unwrap();
        self.stdin.write_all(self.buf.as_bytes())
    }
}

impl Drop for Lemonbar {
    fn drop(&mut self) {
        if let Err(e) = self.proc.kill().and_then(|()| self.proc.wait()) {
            eprintln!("lemonbar kill error: {}", e)
        }
    }
}
