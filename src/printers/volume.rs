use super::printer::{send, PSender, Printer};
use crate::{command::RunelCommand::Set, error::PrinterError, widget::RunelWidget};
use alsa::{
    ctl::Ctl,
    mixer::{Mixer, SelemChannelId, SelemId},
    PollDescriptors,
};
use nix::poll::{poll, PollFd, PollFlags}; // mio?
use std::{
    ffi::CString,
    fmt::{self, Display},
    thread,
};

pub struct Volume;

impl Volume {
    pub fn new() -> Self {
        Self
    }

    fn main_loop(tx: &PSender) -> Result<(), PrinterError> {
        let mut volume_state = VolumeState::new();
        let ctl = Ctl::open(CString::new("default")?.as_ref(), false)?;
        ctl.subscribe_events(true)?;
        let mut fds = ctl
            .get()?
            .iter()
            .map(|f| PollFd::new(f.fd, PollFlags::from_bits_truncate(f.events)))
            .collect::<Vec<_>>();

        volume_state.update_current_state()?;
        send(
            tx,
            Ok(Set {
                widget: RunelWidget::Volume,
                value: volume_state.to_string(),
            }),
        );

        loop {
            poll(&mut fds, -1)?;
            match ctl.read() {
                Ok(Some(_)) => {
                    volume_state.update_current_state()?;
                    send(
                        tx,
                        Ok(Set {
                            widget: RunelWidget::Volume,
                            value: volume_state.to_string(),
                        }),
                    );
                }
                _ => continue,
            }
        }
    }
}

impl Printer for Volume {
    fn spawn(&self, tx: PSender) {
        thread::Builder::new()
            .name("volume".into())
            .spawn(move || {
                if let Err(e) = Self::main_loop(&tx) {
                    send(&tx, Err(e));
                }
            })
            .unwrap();
    }
}

struct VolumeState {
    volume: u8,
    muted: bool,
}

impl VolumeState {
    pub fn new() -> Self {
        Self {
            volume: 0,
            muted: false,
        }
    }

    fn update_current_state(&mut self) -> Result<(), PrinterError> {
        let mixer = Mixer::new("default", false)?;
        let selem_id = SelemId::new("Master", 0);
        if let Some(selem) = mixer.find_selem(&selem_id) {
            let (min, max) = selem.get_playback_volume_range();
            let volume = selem.get_playback_volume(SelemChannelId::FrontLeft)?;
            let volume = (volume as f64 / (max - min) as f64 * 100.0).round() as u8;
            self.volume = volume;

            let switch = selem.get_playback_switch(SelemChannelId::FrontLeft)?;
            self.muted = switch == 0;

            Ok(())
        } else {
            Err(PrinterError::SelemNotFound)
        }
    }
}

impl Display for VolumeState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}",
            self.volume.to_string(),
            if self.muted { "off" } else { "on" }
        )
    }
}
