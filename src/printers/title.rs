use super::printer::{send, CommandPrinter, PSender};
use crate::{command::RunelCommand::Set, error::PrinterError, widget::RunelWidget};
use std::{
    io::{BufRead, BufReader, Error, ErrorKind},
    process::{Child, ChildStdout, Command, Stdio},
    thread,
};

pub struct Title;

impl Title {
    pub fn new() -> Self {
        Self
    }

    fn main_loop<R: BufRead>(tx: &PSender, stdout: R) -> Result<(), PrinterError> {
        for line in stdout.lines() {
            send(
                &tx,
                Ok(Set {
                    widget: RunelWidget::Title,
                    value: line?,
                }),
            );
        }
        Err(PrinterError::Unreachable("title".into()))
    }

    fn start_proc(procs: &mut Vec<Child>) -> Result<BufReader<ChildStdout>, PrinterError> {
        let mut proc = Command::new("xtitle")
            .arg("-siet170")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .spawn()?;

        let stdout = proc
            .stdout
            .take()
            .map(BufReader::new)
            .ok_or_else(|| Error::new(ErrorKind::Other, "could not obtain title's stdout"))?;
        procs.push(proc);

        Ok(stdout)
    }
}

impl CommandPrinter for Title {
    fn spawn(&self, procs: &mut Vec<Child>, tx: PSender) {
        let stdout = match Self::start_proc(procs) {
            Ok(o) => o,
            Err(e) => {
                send(&tx, Err(e));
                return;
            }
        };

        thread::Builder::new()
            .name("title".into())
            .spawn(move || {
                if let Err(e) = Self::main_loop(&tx, stdout) {
                    send(&tx, Err(e));
                }
            })
            .unwrap();
    }
}
