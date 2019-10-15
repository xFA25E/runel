use crate::{command::RunelCommand, path, widget::RunelWidget};

use daemonize::Daemonize;

use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Result as IoResult, Write},
    os::unix::net::UnixStream,
    process::{ChildStdout, Command, Stdio},
};

fn start_daemon() -> Result<(), Box<Error>> {
    Daemonize::new()
        .stderr(File::create(path::mode_err()?)?)
        .stdout(File::create(path::mode_out()?)?)
        .start()?;
    Ok(())
}

pub fn run(command: RunelCommand) -> Result<(), Box<Error>> {
    start_daemon()?;
    let mut stream = UnixStream::connect(path::socket()?)?;

    if !command.is_custom() {
        writeln!(stream, "{}", command)
    } else {
        let mode = path::mode(command.mode_command().unwrap())?;

        let mut cmd = Command::new(mode).stdout(Stdio::piped()).spawn()?;
        let result = send_mode(command, cmd.stdout.take().unwrap(), stream);

        cmd.kill()?;
        result
    }
    .map_err(|e| e.into())
}

fn send_mode(command: RunelCommand, stdout: ChildStdout, stream: UnixStream) -> IoResult<()> {
    let mut stream = BufWriter::new(stream);
    let stdout = BufReader::new(stdout);

    writeln!(stream, "{}", command)?;
    for line in stdout.lines() {
        let cmd = RunelCommand::Set {
            widget: RunelWidget::Custom,
            value: line?,
        };
        writeln!(stream, "{}", cmd)?;
        stream.flush()?;
    }

    Ok(())
}
