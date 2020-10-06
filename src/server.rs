use {
    crate::{
        config::{BSPWM_CMD, CAPACITY, CONFIG_DIR, MAX_MSG_LEN, MQUEUE, TITLE_CMD},
        mode::Mode,
        Colors,
    },
    daemonize::Daemonize,
    posixmq::{unlink, OpenOptions},
    std::{
        error::Error,
        fmt::Write as FmtWrite,
        fs::File,
        io::{self, BufRead, BufReader, Write},
        path::PathBuf,
        process::{Child, ChildStdout, Command, Stdio},
        sync::{mpsc, Arc, RwLock},
        thread,
    },
};

type Sender = mpsc::Sender<Result<(), Arc<io::Error>>>;
type Res<T> = io::Result<T>;
type Value = Arc<RwLock<String>>;
type CmdOut = BufReader<ChildStdout>;

enum Message {
    Ok,
    Quit,
}

enum Update {
    Id(usize),
    Mode(Mode),
}

#[derive(Default)]
struct Bar {
    bspwm: Value,
    title: Value,
    mode: Value,
}

pub fn run(lemonbar_args: Vec<String>, mode: Mode, colors: Colors) -> Result<(), Box<dyn Error>> {
    // _start_daemon()?;

    let mut out = lemonbar_out(lemonbar_args)?;
    let mut buf = String::new();
    let (tx, rx) = mpsc::channel();
    let bar = Bar::default();

    start_bspwm(Arc::clone(&bar.bspwm), tx.clone())?;
    start_title(Arc::clone(&bar.title), tx.clone())?;
    start_listener(Arc::clone(&bar.mode), tx, mode)?;

    for msg in rx {
        msg.map_err(|e| Arc::try_unwrap(e).unwrap())?;
        print_bar(&colors, &mut buf, &bar)?;
        print!("{}", &buf);
        write!(out, "{}", &buf)?;
        out.flush()?;
        buf.clear();
    }
    Ok(())
}

fn lemonbar_out(args: Vec<String>) -> Res<impl Write> {
    let mut child = Command::new("lemonbar")
        .args(&args)
        .stdin(Stdio::piped())
        .spawn()?;

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No stdout of process"))?;

    Ok(stdin)
}

fn start_bspwm(value: Value, tx: Sender) -> Res<()> {
    start_command(value, BSPWM_CMD, tx)
}

fn start_title(value: Value, tx: Sender) -> Res<()> {
    start_command(value, TITLE_CMD, tx)
}

fn print_bar(c: &Colors, out: &mut String, bar: &Bar) -> std::fmt::Result {
    fn split(s: &str) -> Option<(char, &str)> {
        if s.len() > 1 {
            Some((s.as_bytes()[0] as char, &s[1..]))
        } else {
            None
        }
    }

    write!(out, "%{{l}} ")?;
    for (start, name) in bar.bspwm.read().unwrap().split(':').filter_map(split) {
        match start {
            'm' => write!(out, " {}  ", c.monitor.draw(name))?,
            'M' => write!(out, "-{}- ", c.monitor.draw(name))?,
            'f' => write!(out, " {}  ", c.free.draw(name))?,
            'F' => write!(out, "-{}- ", c.free.draw(name))?,
            'o' => write!(out, " {}  ", c.occupied.draw(name))?,
            'O' => write!(out, "-{}- ", c.occupied.draw(name))?,
            'u' => write!(out, " {}  ", c.urgent.draw(name))?,
            'U' => write!(out, "-{}- ", c.urgent.draw(name))?,
            'L' | 'T' | 'G' => write!(out, " {}", c.state.draw(name))?,
            _ => continue,
        }
    }
    writeln!(
        out,
        " {}%{{r}} {} ",
        c.title.draw(bar.title.read().unwrap()),
        bar.mode.read().unwrap()
    )
}

fn update_value(value: &Value, new_value: &str) -> bool {
    let mut value = value.write().unwrap();
    let is_new = *value != new_value;

    if is_new {
        value.clear();
        value.push_str(new_value);
    }

    is_new
}

fn command_stdout(command: &[&str]) -> Res<(Child, CmdOut)> {
    let mut child = Command::new(command[0])
        .args(&command[1..])
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No stdout of process"))?;

    Ok((child, BufReader::new(stdout)))
}

fn start_command(value: Value, command: &[&str], tx: Sender) -> Res<()> {
    let mut buf = String::new();
    let (_, mut stdout) = command_stdout(command)?;

    thread::spawn(move || loop {
        match stdout.read_line(&mut buf) {
            Ok(0) => break,
            Ok(_) => {
                buf.pop();
                if update_value(&value, &buf) {
                    tx.send(Ok(())).unwrap();
                }
                buf.clear();
            }
            Err(e) => {
                tx.send(Err(Arc::new(e))).unwrap();
                break;
            }
        }
    });
    Ok(())
}

fn make_uid() -> impl FnMut() -> usize {
    let mut id = 0;
    move || {
        id += 1;
        id
    }
}

fn start_listener(value: Value, tx: Sender, Mode { mut mode, path }: Mode) -> Res<()> {
    if let Err(_) = unlink(MQUEUE) {}

    let mq = OpenOptions::readonly()
        .max_msg_len(MAX_MSG_LEN)
        .capacity(CAPACITY)
        .create_new()
        .open(MQUEUE)?;
    let mut mq_buffer = [0; MAX_MSG_LEN];

    let mut uid = make_uid();
    let mut id = uid();
    let mut buffer = Arc::new(RwLock::new(String::new()));
    let (mut mtx, mrx) = mpsc::channel();
    let (utx, urx) = mpsc::channel();

    start_mode(path, Arc::clone(&buffer), id, mrx, utx.clone())?;
    thread::spawn({
        let (tx, utx) = (tx.clone(), utx.clone());
        move || loop {
            match mq.receive(&mut mq_buffer) {
                Ok((_, len)) => match String::from_utf8_lossy(&mq_buffer[..len]).parse() {
                    Ok(mode) => utx.send(Ok(Update::Mode(mode))).unwrap(),
                    _ => (),
                },
                Err(e) => {
                    tx.send(Err(Arc::new(e))).unwrap();
                    break;
                }
            }
        }
    });
    thread::spawn(move || {
        for update in urx {
            match update {
                Ok(Update::Mode(Mode { mode: m, path })) if m != mode => {
                    mtx.send(Message::Quit).unwrap();
                    mode = m;
                    buffer = Arc::new(RwLock::new(String::new()));
                    let (mtxn, mrx) = mpsc::channel();
                    mtx = mtxn;
                    id = uid();
                    if let Err(e) = start_mode(path, Arc::clone(&buffer), id, mrx, utx.clone()) {
                        tx.send(Err(Arc::new(e))).unwrap();
                    }
                }
                Ok(Update::Id(i)) if id == i => {
                    if update_value(&value, &buffer.read().unwrap()) {
                        tx.send(Ok(())).unwrap();
                    }
                    mtx.send(Message::Ok).unwrap();
                }
                Err(e) => tx.send(Err(e)).unwrap(),
                Ok(_) => (),
            }
        }
    });

    Ok(())
}

fn start_mode(
    path: PathBuf,
    buf: Value,
    id: usize,
    mode_rx: mpsc::Receiver<Message>,
    update_tx: mpsc::Sender<Result<Update, Arc<io::Error>>>,
) -> Res<()> {
    let (mut child, mut stdout) = command_stdout(&[path.to_str().unwrap()])?;

    thread::spawn(move || {
        loop {
            {
                let mut b = buf.write().unwrap();
                b.clear();
                match stdout.read_line(&mut b) {
                    Ok(0) => break,
                    Ok(_) => {
                        b.pop();
                    }
                    Err(e) => {
                        update_tx.send(Err(Arc::new(e))).unwrap();
                        break;
                    }
                }
            }
            update_tx.send(Ok(Update::Id(id))).unwrap();
            match mode_rx.recv().unwrap() {
                Message::Ok => (),
                Message::Quit => break,
            }
        }
        drop(stdout);
        child.wait().unwrap();
    });
    Ok(())
}

fn _start_daemon() -> Result<(), Box<dyn Error>> {
    let mut path = dirs::runtime_dir().unwrap();
    path.push("test_".to_owned() + CONFIG_DIR);
    let daemon = Daemonize::new().stderr(File::create(path)?);
    daemon.start()?;
    Ok(())
}
