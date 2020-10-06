use {
    crate::{
        config::{BSPWM_CMD, CAPACITY, CONFIG_DIR, MAX_MSG_LEN, MQUEUE, TITLE_CMD},
        mode::Mode,
        Colors,
    },
    daemonize::Daemonize,
    mio::{event::Event, unix::SourceFd, Events, Interest, Poll, Token},
    posixmq::{unlink, OpenOptions, PosixMq},
    std::{
        error::Error,
        fmt::Write as FmtWrite,
        fs::File,
        io::{self, BufRead, BufReader, ErrorKind::WouldBlock, Write},
        mem::swap,
        os::unix::io::{AsRawFd, RawFd},
        process::{Child, ChildStdout, Command, Stdio},
    },
};

type ResUp = io::Result<bool>;
type Value = String;
type CmdOut = BufReader<ChildStdout>;

struct Commands {
    bspwm_stdout: CmdOut,
    bspwm_token: Token,
    title_stdout: CmdOut,
    title_token: Token,
}

struct Mq {
    mq: PosixMq,
    buffer: [u8; MAX_MSG_LEN],
    token: Token,
    mode: Mode,
    mode_stdout: CmdOut,
    mode_token: Token,
    mode_fd: RawFd,
    mode_child: Child,
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
    let mut bar = Bar::default();
    let mut uid = make_uid();
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(64);
    let mut commands = start_commands(&poll, &mut uid)?;
    let mut mq = start_listener(&poll, &mut uid, mode)?;
    let mut buffer = String::new();

    loop {
        poll.poll(&mut events, None)?;

        for etoken in events.iter().map(Event::token) {
            let is_new = match () {
                _ if etoken == mq.token => handle_mq(&mut mq, &poll).map(|()| false)?,
                _ if etoken == commands.bspwm_token => {
                    handle_command(&mut commands.bspwm_stdout, &mut bar.bspwm, &mut buffer)?
                }
                _ if etoken == commands.title_token => {
                    handle_command(&mut commands.title_stdout, &mut bar.title, &mut buffer)?
                }
                _ if etoken == mq.mode_token => {
                    handle_command(&mut mq.mode_stdout, &mut bar.mode, &mut buffer)?
                }
                _ => unreachable!(),
            };

            if is_new {
                fill_buffer(&mut buffer, &bar, &colors)?;
                println!("{}", &buffer);
                write!(out, "{}", &buffer)?;
                out.flush()?;
                buffer.clear();
            }
        }
    }
}

fn make_uid() -> impl FnMut() -> usize {
    let mut id = 0;
    move || {
        id += 1;
        id
    }
}

fn handle_command(stdout: &mut CmdOut, value: &mut Value, buffer: &mut String) -> ResUp {
    if 0 == stdout.read_line(buffer)? {
        return Ok(false);
    }
    buffer.pop();
    let is_new = update_value(value, &buffer);
    buffer.clear();
    Ok(is_new)
}

fn handle_mq(mq: &mut Mq, poll: &Poll) -> io::Result<()> {
    let mut new_mode = None;
    loop {
        match mq.mq.receive(&mut mq.buffer) {
            Ok((_, len)) => match String::from_utf8_lossy(&mq.buffer[..len]).parse() {
                Ok(mode @ Mode { .. }) if mode != mq.mode => new_mode = Some(mode),
                _ => (),
            },
            Err(e) if e.kind() != WouldBlock => return Err(e),
            _ => break,
        }
    }

    if let Some(mode) = new_mode {
        poll.registry().deregister(&mut SourceFd(&mq.mode_fd))?;

        let (stdout, mut child) = command_stdout(&[mode.path.to_str().unwrap()])?;
        swap(&mut mq.mode_child, &mut child);
        mq.mode_fd = stdout.as_raw_fd();
        mq.mode_stdout = BufReader::new(stdout);
        std::thread::spawn(move || term(child));
        mq.mode = mode;
        poll.registry().register(
            &mut SourceFd(&mq.mode_fd),
            mq.mode_token,
            Interest::READABLE,
        )?;
    }

    Ok(())
}

fn start_commands(poll: &Poll, mut uid: impl FnMut() -> usize) -> io::Result<Commands> {
    let mut make = |cmd| -> io::Result<(CmdOut, Token)> {
        let (stdout, _) = command_stdout(cmd)?;
        let token = Token(uid());
        let fd = stdout.as_raw_fd();
        let stdout = BufReader::new(stdout);
        poll.registry()
            .register(&mut SourceFd(&fd), token, Interest::READABLE)?;
        Ok((stdout, token))
    };

    let (bspwm_stdout, bspwm_token) = make(BSPWM_CMD)?;
    let (title_stdout, title_token) = make(TITLE_CMD)?;

    Ok(Commands {
        bspwm_stdout,
        bspwm_token,
        title_stdout,
        title_token,
    })
}

fn start_listener(poll: &Poll, mut uid: impl FnMut() -> usize, mode: Mode) -> io::Result<Mq> {
    if let Err(_) = unlink(MQUEUE) {}

    let token = Token(uid());
    let buffer = [0; MAX_MSG_LEN];
    let mq = OpenOptions::readonly()
        .max_msg_len(MAX_MSG_LEN)
        .capacity(CAPACITY)
        .nonblocking()
        .create_new()
        .open(MQUEUE)?;
    poll.registry()
        .register(&mut SourceFd(&mq.as_raw_fd()), token, Interest::READABLE)?;

    let (mode_stdout, mode_child) = command_stdout(&[mode.path.to_str().unwrap()])?;
    let mode_token = Token(uid());
    let mode_fd = mode_stdout.as_raw_fd();
    let mode_stdout = BufReader::new(mode_stdout);
    poll.registry()
        .register(&mut SourceFd(&mode_fd), mode_token, Interest::READABLE)?;

    Ok(Mq {
        mq,
        buffer,
        token,
        mode,
        mode_stdout,
        mode_token,
        mode_fd,
        mode_child,
    })
}

fn lemonbar_out(args: Vec<String>) -> io::Result<impl Write> {
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

fn command_stdout(command: &[&str]) -> io::Result<(ChildStdout, Child)> {
    let mut child = Command::new(command[0])
        .args(&command[1..])
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No stdout of process"))?;

    Ok((stdout, child))
}

fn update_value(value: &mut Value, new_value: &str) -> bool {
    let is_new = *value != new_value;
    if is_new {
        value.clear();
        value.push_str(new_value)
    }
    is_new
}

fn fill_buffer(buffer: &mut String, bar: &Bar, c: &Colors) -> Result<(), std::fmt::Error> {
    write!(buffer, "%{{l}} ")?;
    for (start, name) in bar.bspwm.split(':').filter_map(split) {
        match start {
            'm' => write!(buffer, " {}  ", c.monitor.draw(name))?,
            'M' => write!(buffer, "-{}- ", c.monitor.draw(name))?,
            'f' => write!(buffer, " {}  ", c.free.draw(name))?,
            'F' => write!(buffer, "-{}- ", c.free.draw(name))?,
            'o' => write!(buffer, " {}  ", c.occupied.draw(name))?,
            'O' => write!(buffer, "-{}- ", c.occupied.draw(name))?,
            'u' => write!(buffer, " {}  ", c.urgent.draw(name))?,
            'U' => write!(buffer, "-{}- ", c.urgent.draw(name))?,
            'L' | 'T' | 'G' => write!(buffer, " {}", c.state.draw(name))?,
            _ => continue,
        }
    }
    write!(buffer, " {}%{{r}} ", c.title.draw(&bar.title))?;
    write!(buffer, "{} ", bar.mode)
}

fn split(s: &str) -> Option<(char, &str)> {
    if s.len() > 1 {
        Some((s.as_bytes()[0] as char, &s[1..]))
    } else {
        None
    }
}

fn term(mut child: Child) {
    // if let Err(e) = kill(Pid::from_raw(child.id() as i32), SIGTERM) {
    //     eprintln!("{}", e);
    // } else
    if let Err(e) = child.wait() {
        eprintln!("{}", e);
    }
}

fn _start_daemon() -> Result<(), Box<dyn Error>> {
    let mut path = dirs::runtime_dir().unwrap();
    path.push("test_".to_owned() + CONFIG_DIR);
    let daemon = Daemonize::new().stderr(File::create(path)?);
    daemon.start()?;
    Ok(())
}
