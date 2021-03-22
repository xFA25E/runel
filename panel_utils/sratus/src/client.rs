use {
    crate::{
        config::{MAX_MSG_LEN, MQUEUE},
        field::Field,
    },
    posixmq::OpenOptions,
    std::io::{
        ErrorKind::{NotFound, WouldBlock},
        Write,
    },
};

pub fn run(field: Field, mut value: String) -> std::io::Result<()> {
    let mq = match OpenOptions::writeonly().nonblocking().open(MQUEUE) {
        Ok(mq) => mq,
        Err(e) if e.kind() == NotFound => return Ok(()),
        Err(e) => return Err(e),
    };
    let field_len = field.as_ref().len();

    value.truncate(MAX_MSG_LEN - field_len);

    let mut buf = Vec::with_capacity(field_len + value.len());
    write!(buf, "{}{}", field, value)?;

    match mq.send(0, &buf) {
        Err(e) if e.kind() == WouldBlock => Ok(()),
        other => other,
    }
}
