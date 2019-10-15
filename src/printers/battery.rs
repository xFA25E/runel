use super::printer::{send, PSender, Printer};
use crate::{command::RunelCommand::Set, widget::RunelWidget};
use std::{fs::read_to_string, io::Error, thread};

pub struct Battery(u64);

impl Battery {
    pub fn new(seconds: u64) -> Self {
        Self(seconds)
    }

    fn main_loop(tx: &PSender, seconds: u64) -> Result<(), Error> {
        loop {
            let status = read_to_string("/sys/class/power_supply/BAT0/status")?;
            let capacity = read_to_string("/sys/class/power_supply/BAT0/capacity")?;

            send(
                tx,
                Ok(Set {
                    widget: RunelWidget::Battery,
                    value: String::from(status.trim()) + " " + capacity.trim(),
                }),
            );
            thread::sleep(std::time::Duration::from_secs(seconds));
        }
    }
}

impl Printer for Battery {
    fn spawn(&self, tx: PSender) {
        let seconds = self.0;

        thread::Builder::new()
            .name("battery".into())
            .spawn(move || {
                if let Err(e) = Self::main_loop(&tx, seconds) {
                    send(&tx, Err(e.into()));
                }
            })
            .unwrap();
    }
}
