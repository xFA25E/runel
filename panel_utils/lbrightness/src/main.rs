use {
    inotify::{EventMask, Inotify, WatchMask},
    std::fs::File,
    std::io::{stdout, BufWriter, Read, Write},
};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

const BRIGHTNESS: &'static str = "/sys/class/backlight/intel_backlight/brightness";
const MAX_BRIGHTNESS: &'static str = "/sys/class/backlight/intel_backlight/max_brightness";

fn read_number(file: &str, buf: &mut String) -> Res<f64> {
    buf.clear();
    File::open(file)?.read_to_string(buf)?;
    buf.pop();
    Ok(buf.parse()?)
}

fn current_brightness(buf: &mut String) -> Res<u8> {
    let cur_br: f64 = read_number(BRIGHTNESS, buf)?;
    let max_br: f64 = read_number(MAX_BRIGHTNESS, buf)?;
    Ok((cur_br / max_br * 100.0).round() as u8)
}

fn run() -> Res<()> {
    let stdout = stdout();
    let mut stdout = BufWriter::new(stdout.lock());
    let mut buf = String::new();
    let mut current = 0;
    let mut show = || -> Res<()> {
        let new = current_brightness(&mut buf)?;
        Ok(if current != new {
            current = new;
            writeln!(stdout, "{}", current)?;
            stdout.flush()?
        })
    };

    show()?;

    loop {
        let mut inotify = Inotify::init()?;
        let current_dir = "/sys/class/backlight/intel_backlight";

        inotify.add_watch(current_dir, WatchMask::MODIFY)?;

        let mut buffer = [0u8; 4096];

        loop {
            for event in inotify.read_events_blocking(&mut buffer)? {
                if event.mask.contains(EventMask::MODIFY) && !event.mask.contains(EventMask::ISDIR)
                {
                    if let Some(n) = event.name.and_then(|f| f.to_str()) {
                        if n == "brightness" {
                            show()?;
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
