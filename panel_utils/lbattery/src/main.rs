use std::{
    env::args,
    io::{stdout, BufWriter, Write},
    {
        fs::File,
        io::{Read, Result},
        thread,
    },
};

const DEFAULT_SECONDS: u64 = 15;
const STATUS: &'static str = "/sys/class/power_supply/BAT0/status";
const CAPACITY: &'static str = "/sys/class/power_supply/BAT0/capacity";

fn run(seconds: u64) -> Result<()> {
    let out = stdout();
    let mut out = BufWriter::new(out.lock());
    let mut buf = String::new();
    let mut new_buf = String::new();

    loop {
        File::open(STATUS)?.read_to_string(&mut new_buf)?;
        new_buf.pop();
        match new_buf.as_str() {
            "Discharging" => {
                new_buf.clear();
                new_buf.push('-')
            }
            "Charging" => {
                new_buf.clear();
                new_buf.push('+')
            }
            "Unknown" => {
                new_buf.clear();
                new_buf.push('?')
            }
            "Full" => {
                new_buf.clear();
                new_buf.push('=')
            }
            _ => new_buf.push(' '),
        }

        File::open(CAPACITY)?.read_to_string(&mut new_buf)?;
        if new_buf != buf {
            std::mem::swap(&mut buf, &mut new_buf);
            write!(out, "{}", &buf)?;
            out.flush()?;
        }
        new_buf.clear();
        thread::sleep(std::time::Duration::from_secs(seconds));
    }
}

fn main() {
    match args()
        .nth(1)
        .map_or_else(|| Ok(DEFAULT_SECONDS), |s| s.parse())
    {
        Ok(seconds) => {
            if let Err(e) = run(seconds) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
