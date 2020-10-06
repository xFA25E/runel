use {
    crate::{
        config::{CAPACITY, MAX_MSG_LEN, MQUEUE},
        mode::Mode,
    },
    posixmq::OpenOptions,
    std::io::ErrorKind::WouldBlock,
};

pub fn run(mode: Mode) -> std::io::Result<()> {
    let mq = OpenOptions::writeonly()
        .max_msg_len(MAX_MSG_LEN)
        .capacity(CAPACITY)
        .nonblocking()
        .create()
        .open(MQUEUE)?;

    let mut mode = mode.mode;
    mode.truncate(MAX_MSG_LEN);

    match mq.send(0, mode.as_bytes()) {
        Err(e) if e.kind() == WouldBlock => Ok(()),
        other => other,
    }
}
