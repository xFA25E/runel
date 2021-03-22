use {
    crate::{
        config::{CAPACITY, FIELDS, MAX_MSG_LEN, MQUEUE},
        Colors,
    },
    nix::{
        sys::signal::{kill, Signal::SIGTERM},
        unistd::Pid,
    },
    posixmq::{unlink, OpenOptions},
    simple_signal::Signal,
    std::{
        collections::HashMap,
        io::{self, BufRead, BufReader, BufWriter, Write},
        process::{Child, ChildStdout, Command, Stdio},
        sync::{mpsc, Arc, RwLock},
        thread,
    },
};

type Fields = Arc<HashMap<&'static str, Arc<RwLock<String>>>>;
type Sender = mpsc::Sender<Result<(), Arc<io::Error>>>;
type Res<T> = io::Result<T>;
type Value = Arc<RwLock<String>>;
type CmdOut = BufReader<ChildStdout>;

pub fn run(colors: Colors) -> Res<()> {
    let stdout = io::stdout();
    let mut stdout = BufWriter::new(stdout.lock());
    let (tx, rx) = mpsc::channel();
    let fields = create_fields();

    let children = start_commands(&fields, &tx)?;
    simple_signal::set_handler(&[Signal::Term, Signal::Int], move |_| {
        for child in &children {
            if let Err(e) = kill(Pid::from_raw(child.id() as i32), SIGTERM) {
                eprintln!("{}", e);
            }
        }
        std::process::exit(0);
    });

    start_listener(Arc::clone(&fields), tx)?;

    for msg in rx {
        msg.map_err(|e| Arc::try_unwrap(e).unwrap())?;
        print_fields(&colors, &mut stdout, &fields)?;
    }
    Ok(())
}

fn create_fields() -> Fields {
    let mut fields: HashMap<&'static str, _> = HashMap::with_capacity(FIELDS.len());
    for (field, _, _, default) in FIELDS {
        let default = Arc::new(RwLock::new(default.to_string()));
        fields.insert(field, Arc::clone(&default));
    }
    Arc::new(fields)
}

fn start_commands(fields: &Fields, tx: &Sender) -> Res<Vec<Child>> {
    let mut children = Vec::with_capacity(FIELDS.iter().filter(|f| f.2.is_some()).count());
    for (field, _, command, _) in FIELDS {
        if let Some(command) = command {
            let value = Arc::clone(fields.get(field).unwrap());
            children.push(start_command(value, command, tx.clone())?);
        }
    }
    Ok(children)
}

fn print_fields(c: &Colors, stdout: &mut dyn Write, fields: &Fields) -> Res<()> {
    let mut iter = FIELDS.iter().map(|f| (f.0, f.1));
    let get_field = |f| fields.get(f).unwrap().read().unwrap();

    if let Some((field, hd)) = iter.next() {
        let (head, body) = (c.head.draw(hd), c.body.draw(get_field(field)));
        write!(stdout, "{}  {}", head, body)?;
    }

    for (field, hd) in iter {
        let (head, body) = (c.head.draw(hd), c.body.draw(get_field(field)));
        write!(stdout, "  {}  {}", head, body)?;
    }

    writeln!(stdout)?;
    stdout.flush()
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

fn find_field<'a>(data: &'a std::borrow::Cow<str>) -> Option<(&'static str, &'a str)> {
    FIELDS
        .iter()
        .map(|f| f.0)
        .find(|f| data.starts_with(f))
        .map(|field| (field, &data[field.len()..]))
}

fn start_listener(fields: Fields, tx: Sender) -> Res<()> {
    if let Err(_) = unlink(MQUEUE) {}

    let mq = OpenOptions::readonly()
        .max_msg_len(MAX_MSG_LEN)
        .capacity(CAPACITY)
        .create_new()
        .open(MQUEUE)?;
    let mut buf = [0; MAX_MSG_LEN];

    thread::spawn(move || loop {
        match mq.receive(&mut buf) {
            Ok((_, len)) => {
                let data = String::from_utf8_lossy(&buf[..len]);

                if let Some((field, data)) = find_field(&data) {
                    let mut value = fields.get(field).unwrap().write().unwrap();

                    if *value != data {
                        value.clear();
                        value.push_str(data);
                        tx.send(Ok(())).unwrap();
                    }
                }
            }
            Err(e) => {
                tx.send(Err(Arc::new(e))).unwrap();
                break;
            }
        }
    });

    Ok(())
}
