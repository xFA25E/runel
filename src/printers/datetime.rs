use super::printer::{send, PSender, Printer};
use crate::{command::RunelCommand::Set, widget::RunelWidget};
use chrono::Local;
use std::{thread, time};

pub struct DateTime<'b> {
    seconds: u64,
    widget: RunelWidget,
    format: &'b str,
}

impl<'b> DateTime<'b> {
    pub fn new(seconds: u64, widget: RunelWidget, format: &'b str) -> Self {
        Self {
            seconds,
            widget,
            format,
        }
    }
}

impl<'b> Printer for DateTime<'b> {
    fn spawn(&self, tx: PSender) {
        let seconds = self.seconds;
        let format = String::from(self.format);
        let widget = self.widget;

        thread::Builder::new()
            .name(widget.to_string())
            .spawn(move || loop {
                send(
                    &tx,
                    Ok(Set {
                        widget,
                        value: Local::now().format(&format).to_string(),
                    }),
                );
                thread::sleep(time::Duration::from_secs(seconds));
            })
            .unwrap();
    }
}
