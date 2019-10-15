use super::printer::{send, PSender, Printer};
use crate::{command::RunelCommand, error::PrinterError, path};
use std::{
    io::{BufRead, BufReader, Result as IoResult, Write},
    os::unix::net::{UnixListener, UnixStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Sender},
        Arc,
    },
    thread,
};

pub struct Socket;

enum SocketMsg {
    New(UnixStream),
    Line(String),
}
type SSender = Sender<IoResult<SocketMsg>>;
type Abool = Arc<AtomicBool>;

impl Socket {
    pub fn new() -> Self {
        Self
    }

    fn handle_connection(sock: UnixStream, thread_tx: SSender, quit: Abool, error_tx: PSender) {
        thread::spawn(move || {
            for line in BufReader::new(sock).lines() {
                if quit.load(Ordering::Relaxed) {
                    break;
                }
                if let Err(e) = thread_tx.send(line.map(SocketMsg::Line)) {
                    let err = e.to_string() + ": socket connection";
                    send(&error_tx, Err(PrinterError::Unreachable(err)));
                    break;
                }
            }
        });
    }

    fn start_listener(listener: UnixListener, main_tx: SSender, error_tx: PSender) {
        thread::spawn(move || {
            let mut err = String::from("socket listener: ");
            for stream in listener.incoming() {
                if let Err(e) = main_tx.send(stream.map(SocketMsg::New)) {
                    err += &e.to_string();
                    break;
                }
            }
            send(&error_tx, Err(PrinterError::Unreachable(err)))
        });
    }

    fn main_loop(tx: &PSender) -> Result<(), PrinterError> {
        let path = path::socket()?;
        if path.exists() {
            if let Ok(mut stream) = UnixStream::connect(&path) {
                writeln!(stream, "{}", RunelCommand::Quit)?;
            }
            std::fs::remove_file(&path)?;
        }
        let (main_tx, main_rx) = mpsc::channel();
        let thread_tx = main_tx.clone();
        Self::start_listener(UnixListener::bind(&path)?, main_tx, tx.clone());

        let mut quit = Arc::new(AtomicBool::new(false));

        for msg in main_rx {
            match msg? {
                SocketMsg::New(sock) => {
                    quit.store(true, Ordering::Relaxed);
                    quit = Arc::new(AtomicBool::new(false));
                    Self::handle_connection(sock, thread_tx.clone(), quit.clone(), tx.clone());
                }
                SocketMsg::Line(line) => {
                    send(tx, Ok(line.parse::<RunelCommand>()?));
                }
            }
        }

        Err(PrinterError::Unreachable("socket".into()))
    }
}

impl Printer for Socket {
    fn spawn(&self, tx: PSender) {
        thread::Builder::new()
            .name("socket".into())
            .spawn(move || {
                if let Err(e) = Self::main_loop(&tx) {
                    send(&tx, Err(e));
                }
            })
            .unwrap();
    }
}
