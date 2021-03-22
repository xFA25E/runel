use chrono::Local;
use std::{
    fmt::Write as FmtWrite,
    io::{stdout, BufWriter, Result, Write},
};

fn run(seconds: u64, format: String) -> Result<()> {
    let out = stdout();
    let mut out = BufWriter::new(out.lock());
    let mut buf = String::new();
    let mut new_buf = String::new();

    loop {
        if let Err(_) = write!(new_buf, "{}", Local::now().format(&format)) {}
        if new_buf != buf {
            std::mem::swap(&mut buf, &mut new_buf);
            writeln!(out, "{}", &buf)?;
            out.flush()?;
        }
        new_buf.clear();
        std::thread::sleep(std::time::Duration::from_secs(seconds));
    }
}

fn main() {
    let mut args = std::env::args();
    args.next();

    let seconds = match args.next() {
        Some(seconds) => match seconds.parse() {
            Ok(seconds) => seconds,
            Err(e) => return eprintln!("{}", e),
        },
        None => return eprintln!("Require at least 2 arguments"),
    };

    let format = match args.next() {
        Some(format) => format,
        None => return eprintln!("Require at least 2 arguments"),
    };

    if let Err(e) = run(seconds, format) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
