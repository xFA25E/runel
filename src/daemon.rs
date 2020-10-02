use crate::{
    colors::Colors, command::RunelCommand, lemonbar::Lemonbar, panel::Panel, path,
    printers::Printers,
};
use daemonize::Daemonize;
use std::{error::Error, fs::File};

macro_rules! debug_log {
    ($val:expr) => {
        if cfg!(debug_assertions) {
            println!("{}", $val);
        }
    };
}

pub fn run(lemonbar_args: Vec<String>) -> Result<(), Box<dyn Error>> {
    start_daemon()?;

    let mut lemonbar = Lemonbar::start(lemonbar_args)?;
    let mut panel = Panel::new(Colors::new_from_xresources()?);
    let mut printers = Printers::new();
    let rx = printers.spawn();

    for rcmd in rx {
        let rcmd = rcmd?;
        debug_log!(rcmd);

        let status = match rcmd {
            RunelCommand::Set { widget, value } => panel.set(widget, value),
            RunelCommand::Mode { mode, .. } => panel.mode(mode),
            RunelCommand::Reload => panel.reload()?,
            RunelCommand::Quit => break,
        };
        if status.is_updated() {
            debug_log!(panel);
            lemonbar.send(&panel)?;
        }
    }

    Ok(())
}

fn start_daemon() -> Result<(), Box<dyn Error>> {
    let pid_path = path::pid()?;
    if pid_path.exists() {
        std::fs::remove_file(&pid_path)?;
    }

    let mut daemon = Daemonize::new()
        .pid_file(pid_path)
        .stderr(File::create(path::daemon_err()?)?);

    if cfg!(debug_assertions) {
        daemon = daemon.stdout(File::create(path::daemon_out()?)?);
    }
    daemon.start()?;
    Ok(())
}
