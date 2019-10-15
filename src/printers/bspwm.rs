use super::printer::{send, PSender, Printer};
use crate::{command::RunelCommand::Set, error::PrinterError, widget::RunelWidget};
use std::{
    env,
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    thread,
};

pub struct Bspwm;

impl Bspwm {
    pub fn new() -> Self {
        Self
    }

    fn main_loop(tx: &PSender) -> Result<(), PrinterError> {
        let bspwm_socket = env::var("BSPWM_SOCKET")
            .map(PathBuf::from)
            .or_else(|_| Self::find_socket())?;

        let mut stream = UnixStream::connect(&bspwm_socket)?;
        write!(stream, "subscribe\x00report\x00")?;

        for line in BufReader::new(stream).lines() {
            send(
                tx,
                Ok(Set {
                    widget: RunelWidget::Bspwm,
                    value: line?,
                }),
            );
        }
        Err(PrinterError::Unreachable("bspwm".into()))
    }

    fn find_socket() -> Result<PathBuf, PrinterError> {
        let regex = regex::Regex::new(r"bspwm(\w+)?_(\d+)?_(\d+)?-socket")?;

        for entry in Path::new("/tmp").read_dir()? {
            if let Some(entry) = entry?.path().to_str() {
                if regex.is_match(entry) {
                    return Ok(PathBuf::from(entry));
                }
            }
        }
        Err(PrinterError::BspwmNotFound)
    }
}

impl Printer for Bspwm {
    fn spawn(&self, tx: PSender) {
        thread::Builder::new()
            .name("bspwm".into())
            .spawn(move || {
                if let Err(e) = Self::main_loop(&tx) {
                    send(&tx, Err(e));
                }
            })
            .unwrap();
    }
}
