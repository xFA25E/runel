use crate::{command::RunelCommand, error::PrinterError};
use std::{
    process::Child,
    sync::mpsc::{Receiver, Sender},
};

type PResult = Result<RunelCommand, PrinterError>;
pub type PSender = Sender<PResult>;
pub type PReceiver = Receiver<PResult>;

pub trait Printer {
    fn spawn(&self, tx: PSender);
}

pub trait CommandPrinter {
    fn spawn(&self, procs: &mut Vec<Child>, tx: PSender);
}

pub fn send(tx: &PSender, s: PResult) {
    tx.send(s).expect("On sending line through channel")
}
