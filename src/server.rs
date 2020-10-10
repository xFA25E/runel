use {
    crate::{
        config::{CAPACITY, CONFIG_DIR, MAX_MSG_LEN, MQUEUE, TITLE_CMD, WMSTATUS_CMD},
        mode::Mode,
        Color,
    },
    daemonize::Daemonize,
    nix::{
        sys::signal::{kill, Signal::SIGTERM},
        unistd::Pid,
    },
    posixmq::{unlink, OpenOptions},
    signal_msg::{Signal, SignalReceiver, SignalSender},
    std::{
        collections::HashMap,
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

enum KillerMessage {
    Child(usize, Child),
    Kill(usize),
    Signal,
}

enum Update {
    Id(usize),
    Mode(Mode),
}

#[derive(Default)]
struct Bar {
    wmstatus: Value,
    title: Value,
    mode: Value,
}

pub fn run(lemonbar_args: Vec<String>, mode: Mode, title: Color) -> Result<(), Box<dyn Error>> {
    start_daemon()?;

    let mut out = lemonbar_out(lemonbar_args)?;
    let mut buf = String::new();
    let (tx, rx) = mpsc::channel();
    let bar = Bar::default();

    let wmstatus_child = start_command(Arc::clone(&bar.wmstatus), WMSTATUS_CMD, tx.clone())?;
    let title_child = start_command(Arc::clone(&bar.title), TITLE_CMD, tx.clone())?;
    let killer_tx = start_child_killer(vec![wmstatus_child, title_child])?;
    start_listener(Arc::clone(&bar.mode), tx, mode, killer_tx)?;

    for msg in rx {
        msg.map_err(|e| Arc::try_unwrap(e).unwrap())?;
        print_bar(&title, &mut buf, &bar)?;
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

fn print_bar(title: &Color, out: &mut String, bar: &Bar) -> std::fmt::Result {
    writeln!(
        out,
        "%{{l}} {} {}%{{r}} {} ",
        bar.wmstatus.read().unwrap(),
        title.draw(bar.title.read().unwrap()),
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

fn start_command(value: Value, command: &[&str], tx: Sender) -> Res<Child> {
    let mut new_buf = String::new();
    let (child, mut stdout) = command_stdout(command)?;

    thread::spawn(move || loop {
        match stdout.read_line(&mut new_buf) {
            Ok(0) => break,
            Ok(_) => {
                new_buf.pop();
                let mut buf = value.write().unwrap();
                if new_buf != *buf {
                    std::mem::swap(&mut *buf, &mut new_buf);
                    tx.send(Ok(())).unwrap();
                }
                new_buf.clear();
            }
            Err(e) => {
                tx.send(Err(Arc::new(e))).unwrap();
                break;
            }
        }
    });
    Ok(child)
}

fn make_uid() -> impl FnMut() -> usize {
    let mut id: usize = 0;
    move || {
        id = id.overflowing_add(1).0;
        id
    }
}

fn start_listener(
    value: Value,
    tx: Sender,
    Mode { mut mode, path }: Mode,
    killer_tx: mpsc::Sender<KillerMessage>,
) -> Res<()> {
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

    start_mode(
        path,
        Arc::clone(&buffer),
        id,
        mrx,
        utx.clone(),
        killer_tx.clone(),
    )?;
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
                    if let Err(_) = mtx.send(Message::Quit) {}
                    mode = m;
                    buffer = Arc::new(RwLock::new(String::new()));
                    let (mtxn, mrx) = mpsc::channel();
                    mtx = mtxn;
                    id = uid();
                    if let Err(e) = start_mode(
                        path,
                        Arc::clone(&buffer),
                        id,
                        mrx,
                        utx.clone(),
                        killer_tx.clone(),
                    ) {
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
    killer_tx: mpsc::Sender<KillerMessage>,
) -> Res<()> {
    let (child, mut stdout) = command_stdout(&[path.to_str().unwrap()])?;
    killer_tx.send(KillerMessage::Child(id, child)).unwrap();

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
        killer_tx.send(KillerMessage::Kill(id)).unwrap();
    });
    Ok(())
}

fn kill_child(mut child: Child) {
    if let Err(e) = kill(Pid::from_raw(child.id() as i32), SIGTERM) {
        eprintln!("{}", e);
    } else {
        std::thread::sleep(std::time::Duration::from_secs(1));
        match child.try_wait() {
            Ok(None) => {
                if let Err(e) = child.kill() {
                    eprintln!("{}", e);
                } else if let Err(e) = child.wait() {
                    eprintln!("{}", e);
                }
            }
            Err(e) => eprintln!("{}", e),
            _ => (),
        }
    }
}

fn start_child_killer(mut children: Vec<Child>) -> Res<mpsc::Sender<KillerMessage>> {
    let (killer_tx, killer_rx) = mpsc::channel();
    let mut status_children: HashMap<usize, Child> = HashMap::new();

    thread::spawn({
        let killer_tx = killer_tx.clone();
        move || {
            let (signal_sender, signal_receiver) = signal_msg::new();
            signal_sender.prepare_signals();

            loop {
                match signal_receiver.listen() {
                    Ok(Signal::Term) | Ok(Signal::Int) => {
                        killer_tx.send(KillerMessage::Signal).unwrap()
                    }
                    _ => (),
                }
            }
        }
    });

    thread::spawn(move || {
        for msg in killer_rx {
            match msg {
                KillerMessage::Signal => {
                    for child in children.drain(..) {
                        if let Err(e) = kill(Pid::from_raw(child.id() as i32), SIGTERM) {
                            eprintln!("{}", e);
                        }
                    }
                    for (_, child) in status_children.drain() {
                        if let Err(e) = kill(Pid::from_raw(child.id() as i32), SIGTERM) {
                            eprintln!("{}", e);
                        }
                    }
                    std::process::exit(0);
                }
                KillerMessage::Child(id, child) => {
                    status_children.insert(id, child);
                }
                KillerMessage::Kill(id) => {
                    status_children.remove(&id).map(kill_child);
                }
            }
        }
    });

    Ok(killer_tx)
}

fn start_daemon() -> Result<(), Box<dyn Error>> {
    let mut path = dirs::runtime_dir().unwrap();
    path.push(CONFIG_DIR);
    let daemon = Daemonize::new().stderr(File::create(path)?);
    daemon.start()?;
    Ok(())
}
