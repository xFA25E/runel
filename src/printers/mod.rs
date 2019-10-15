mod battery;
mod brightness;
mod bspwm;
mod datetime;
mod keyboard;
mod printer;
mod socket;
mod title;
mod volume;

use self::{
    battery::Battery,
    brightness::Brightness,
    bspwm::Bspwm,
    datetime::DateTime,
    keyboard::Keyboard,
    printer::{CommandPrinter, PReceiver, Printer},
    socket::Socket,
    title::Title,
    volume::Volume,
};
use crate::widget::RunelWidget;
use std::{
    process::Child,
    sync::mpsc::{channel, Sender},
};

#[derive(Debug)]
pub struct Printers {
    procs: Vec<Child>,
}

impl Printers {
    pub fn new() -> Self {
        Printers { procs: Vec::new() }
    }

    pub fn spawn(&mut self) -> PReceiver {
        let (tx, rx) = channel();

        let printers: Vec<Box<Printer>> = vec![
            Box::new(Battery::new(15)),
            Box::new(Brightness::new()),
            Box::new(Bspwm::new()),
            Box::new(DateTime::new(3, RunelWidget::Clock, "%R")),
            Box::new(DateTime::new(60, RunelWidget::Date, "%d %b, %a")),
            Box::new(Volume::new()),
            Box::new(Socket::new()),
            Box::new(Keyboard::new()),
        ];

        printers.iter().for_each(|p| {
            let t = Sender::clone(&tx);
            p.spawn(t);
        });

        Title::new().spawn(&mut self.procs, tx);

        rx
    }
}

impl Drop for Printers {
    fn drop(&mut self) {
        for p in self.procs.iter_mut() {
            if let Err(e) = p.kill().and_then(|_| p.wait()) {
                eprintln!("error while killing process: {}", e);
            }
        }
    }
}
