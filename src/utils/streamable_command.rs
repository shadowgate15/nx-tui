use color_eyre::Result;
use futures::StreamExt;
use std::{ffi::OsStr, fmt, pin::Pin, process::Stdio};
use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    sync::mpsc,
};
use tokio_stream::{wrappers::LinesStream, Stream, StreamExt, StreamMap};
use tracing::debug;

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum IoType {
    StdOut,
    StdErr,
}

impl fmt::Display for IoType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IoType::StdOut => write!(f, "stdout"),
            IoType::StdErr => write!(f, "stderr"),
        }
    }
}

pub struct Command {
    tokio: tokio::process::Command,
    command_log_tx: Option<mpsc::UnboundedSender<String>>,
}

impl Command {
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        Self::from(tokio::process::Command::new(program))
    }

    pub fn command_log_tx(&mut self, command_log_tx: mpsc::UnboundedSender<String>) -> &mut Self {
        self.command_log_tx = Some(command_log_tx);
        self
    }

    pub fn stream<'a>(&'a mut self) -> Result<impl Stream<Item = (IoType, String)> + Send + 'a> {
        let mut map = StreamMap::new();
        let mut child = self
            .tokio
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(stdout) = child.stdout.take() {
            let stdout = LinesStream::new(BufReader::new(stdout).lines())
                .map(|next| {
                    let line = next.unwrap();

                    debug!("[{}] {}", IoType::StdOut, line);

                    if let Some(command_log_tx) = &self.command_log_tx {
                        let _ = command_log_tx.send(line.clone());
                    }

                    line
                })
                .boxed();

            map.insert(IoType::StdOut, stdout);
        }
        if let Some(stderr) = child.stderr.take() {
            let stderr = LinesStream::new(BufReader::new(stderr).lines())
                .map(|next| {
                    let line = next.unwrap();

                    debug!("[{}] {}", IoType::StdErr, line);

                    if let Some(command_log_tx) = &self.command_log_tx {
                        let _ = command_log_tx.send(line.clone());
                    }

                    line
                })
                .boxed();

            map.insert(IoType::StdErr, stderr);
        }

        Ok(map)
    }
}

impl From<tokio::process::Command> for Command {
    fn from(command: tokio::process::Command) -> Self {
        Self {
            tokio: command,
            command_log_tx: None,
        }
    }
}

impl std::ops::Deref for Command {
    type Target = tokio::process::Command;

    fn deref(&self) -> &Self::Target {
        &self.tokio
    }
}

impl std::ops::DerefMut for Command {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tokio
    }
}
