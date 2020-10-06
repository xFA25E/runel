use {
    crate::{
        config::{MAX_MSG_LEN, MQUEUE},
        mode::Mode,
    },
    posixmq::OpenOptions,
    std::io::ErrorKind::{NotFound, WouldBlock},
};

pub fn run(mode: Mode) -> std::io::Result<()> {
    let mq = match OpenOptions::writeonly().nonblocking().open(MQUEUE) {
        Ok(mq) => mq,
        Err(e) if e.kind() == NotFound => return Ok(()),
        Err(e) => return Err(e),
    };

    let mut mode = mode.mode;
    mode.truncate(MAX_MSG_LEN);

    match mq.send(0, mode.as_bytes()) {
        Err(e) if e.kind() == WouldBlock => Ok(()),
        other => other,
    }
}
