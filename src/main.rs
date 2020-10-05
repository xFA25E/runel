mod client;
mod color;
mod config;
mod mode;
mod server;

use {color::Color, mode::Mode, structopt::StructOpt};

#[derive(StructOpt)]
pub struct Colors {
    #[structopt(long = "color-free", name = "COLOR_FREE", default_value = "")]
    /// A color for free desktop
    free: Color,
    #[structopt(long = "color-monitor", name = "COLOR_MONITOR", default_value = "")]
    /// A color for monitor
    monitor: Color,
    #[structopt(long = "color-occupied", name = "COLOR_OCCUPIED", default_value = "")]
    /// A color for occupied desktop
    occupied: Color,
    #[structopt(long = "color-urgent", name = "COLOR_URGENT", default_value = "")]
    /// A color for urgent desktop
    urgent: Color,
    #[structopt(long = "color-state", name = "COLOR_STATE", default_value = "")]
    /// A color for window state
    state: Color,
    #[structopt(long = "color-title", name = "COLOR_TITLE", default_value = "")]
    /// A color for window title
    title: Color,
}

#[derive(StructOpt)]
/// A Multi-status wrapper for lemonbar
struct Args {
    #[structopt(flatten)]
    colors: Colors,
    #[structopt(short, long, name = "MODE")]
    /// Mode to run
    mode: Mode,
    #[structopt(short, long)]
    /// Start runel server
    server: bool,
    #[structopt(name = "LEMONBAR_ARGS", env, last = true, use_delimiter = true)]
    /// Lemonbar command line arguments
    lemonbar_args: Vec<String>,
}

fn main() {
    let opts = Args::from_args();

    let result = match opts {
        Args {
            server: true,
            mode,
            colors,
            lemonbar_args,
        } => server::run(lemonbar_args, mode, colors),

        Args {
            server: false,
            mode,
            ..
        } => client::run(mode).map_err(|e| e.into()),
    };

    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
