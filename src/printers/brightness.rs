use super::printer::{send, PSender, Printer};
use crate::{command::RunelCommand::Set, error::PrinterError, widget::RunelWidget};
use inotify::{EventMask, Inotify, WatchMask};
use std::{fs::read_to_string, thread};

pub struct Brightness;

impl Brightness {
    pub fn new() -> Self {
        Self
    }

    fn main_loop(tx: &PSender) -> Result<(), PrinterError> {
        let send_brightness = || -> Result<(), PrinterError> {
            let cur_br = Self::current_brightness()?;
            send(
                tx,
                Ok(Set {
                    widget: RunelWidget::Brightness,
                    value: cur_br.to_string(),
                }),
            );
            Ok(())
        };

        send_brightness()?;
        loop {
            let mut inotify = Inotify::init()?;
            let current_dir = "/sys/class/backlight/intel_backlight";

            inotify.add_watch(current_dir, WatchMask::MODIFY)?;

            let mut buffer = [0u8; 4096];

            loop {
                for event in inotify.read_events_blocking(&mut buffer)? {
                    if event.mask.contains(EventMask::MODIFY)
                        && !event.mask.contains(EventMask::ISDIR)
                    {
                        if let Some(n) = event.name.and_then(|f| f.to_str()) {
                            if n == "brightness" {
                                send_brightness()?;
                            }
                        }
                    }
                }
            }
        }
    }

    fn current_brightness() -> Result<u8, PrinterError> {
        let read_num =
            |s| -> Result<f64, PrinterError> { Ok(read_to_string(s)?.trim().parse::<f64>()?) };
        let cur_br: f64 = read_num("/sys/class/backlight/intel_backlight/brightness")?;
        let max_br: f64 = read_num("/sys/class/backlight/intel_backlight/max_brightness")?;
        Ok((cur_br / max_br * 100.0).round() as u8)
    }
}

impl Printer for Brightness {
    fn spawn(&self, tx: PSender) {
        thread::Builder::new()
            .name("brightness".into())
            .spawn(move || {
                if let Err(e) = Self::main_loop(&tx) {
                    send(&tx, Err(e));
                }
            })
            .unwrap();
    }
}
