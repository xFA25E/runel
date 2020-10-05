use {
    crate::{
        config::{CAPACITY, MAX_MSG_LEN, MQUEUE},
        mode::Mode,
    },
    posixmq::OpenOptions,
};

pub fn run(mode: Mode) -> std::io::Result<()> {
    let mq = OpenOptions::writeonly()
        .max_msg_len(MAX_MSG_LEN)
        .capacity(CAPACITY)
        .create()
        .open(MQUEUE)?;

    let mut mode = mode.mode;
    mode.truncate(MAX_MSG_LEN);

    mq.send_timeout(0, mode.as_bytes(), std::time::Duration::from_secs(1))
}
