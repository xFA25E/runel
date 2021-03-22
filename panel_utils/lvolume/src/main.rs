use {
    alsa::{
        ctl::Ctl,
        mixer::{Mixer, SelemChannelId, SelemId},
        PollDescriptors,
    },
    nix::poll::{poll, PollFd, PollFlags}, // mio?
    std::{
        ffi::CString,
        fmt::{self, Display},
        io::{stdout, BufWriter, Write},
    },
};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

fn run() -> Res<()> {
    let stdout = stdout();
    let mut stdout = BufWriter::new(stdout.lock());
    let mut volume_state = VolumeState::new();
    let mut show = || -> Res<()> {
        Ok(if volume_state.update_current_state()? {
            writeln!(stdout, "{}", volume_state)?;
            stdout.flush()?
        })
    };

    let ctl = Ctl::open(CString::new("default")?.as_ref(), false)?;
    ctl.subscribe_events(true)?;
    let mut fds = ctl
        .get()?
        .iter()
        .map(|f| PollFd::new(f.fd, PollFlags::from_bits_truncate(f.events)))
        .collect::<Vec<_>>();

    show()?;

    loop {
        poll(&mut fds, -1)?;
        match ctl.read() {
            Ok(Some(_)) => show()?,
            _ => continue,
        }
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

    fn update_current_state(&mut self) -> Res<bool> {
        let mut is_new = false;
        let mixer = Mixer::new("default", false)?;
        let selem_id = SelemId::new("Master", 0);
        if let Some(selem) = mixer.find_selem(&selem_id) {
            let (min, max) = selem.get_playback_volume_range();
            let volume = selem.get_playback_volume(SelemChannelId::FrontLeft)?;
            let volume = (volume as f64 / (max - min) as f64 * 100.0).round() as u8;
            is_new = is_new || self.volume != volume;
            self.volume = volume;

            let switch = selem.get_playback_switch(SelemChannelId::FrontLeft)?;
            let muted = switch == 0;
            is_new = is_new || self.muted != muted;
            self.muted = muted;
        }
        Ok(is_new)
    }
}

impl Display for VolumeState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.volume)?;
        write!(f, " {}", if self.muted { "off" } else { "on" })
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
