mod client;
mod color;
mod config;
mod mode;
mod server;

use {color::Color, mode::Mode, structopt::StructOpt};

#[derive(StructOpt)]
/// A Multi-status wrapper for lemonbar
struct Args {
    #[structopt(long = "color-title", name = "COLOR_TITLE", default_value = "")]
    /// A color for window title
    title: Color,
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
            title,
            lemonbar_args,
        } => server::run(lemonbar_args, mode, title),

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
