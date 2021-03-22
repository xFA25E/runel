mod client;
mod color;
mod config;
mod field;
mod server;

use {
    color::Color,
    field::Field,
    std::io::{Error, ErrorKind},
    structopt::StructOpt,
};

#[derive(StructOpt)]
/// A status bar for lemonbar
struct Args {
    #[structopt(flatten)]
    colors: Colors,
    #[structopt(short, long, name = "FIELD")]
    /// Update field of statusbar
    update: Option<Field>,
    #[structopt(short, long, name = "VALUE")]
    /// Update value
    value: Option<String>,
}

#[derive(StructOpt)]
pub struct Colors {
    #[structopt(long = "color-head", name = "COLOR_HEAD", default_value = "")]
    /// Color for field head
    head: Color,
    #[structopt(long = "color-body", name = "COLOR_BODY", default_value = "")]
    /// Color for field body
    body: Color,
}

fn main() {
    let opts = Args::from_args();

    let result = match opts {
        // Server
        Args {
            colors,
            update: None,
            value: None,
        } => server::run(colors),

        // Client
        Args {
            update: Some(field),
            value: Some(value),
            ..
        } => client::run(field, value),

        // Args error
        _ => Err(Error::new(
            ErrorKind::Other,
            "Invalid arguments: should I update or exit?",
        )),
    };

    if let Err(e) = result {
        eprintln!("Runtime error: {}", e);
        std::process::exit(1);
    }
}
