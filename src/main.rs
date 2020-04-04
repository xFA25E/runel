#![warn(clippy::all)]

mod colors;
mod command;
mod config;
mod daemon;
mod error;
mod lemonbar;
mod mode;
mod panel;
mod path;
mod printers;
mod remote;
mod widget;
mod widgets;

use crate::config::Config;

fn main() {
    if let Err(e) = run(Config::new()) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    match config {
        Config::Daemon(lemonbar_args) => daemon::run(lemonbar_args),
        Config::Remote(command) => remote::run(command),
    }
}
