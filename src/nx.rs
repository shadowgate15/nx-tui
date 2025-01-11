use std::{
    fmt, io,
    path::PathBuf,
    pin::Pin,
    process::{ExitStatus, Stdio},
};

use color_eyre::Result;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    sync::mpsc,
};
use tokio_stream::{wrappers::LinesStream, Stream, StreamExt, StreamMap};
use tracing::debug;

use crate::{
    action::{Action, ActionReceiver, ActionSender},
    utils::streamable_command::Command,
};

pub struct Output {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

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

pub struct Nx {
    path: PathBuf,
    pub action_tx: ActionSender,
    pub action_rx: ActionReceiver,
    pub command_log_tx: mpsc::UnboundedSender<String>,
}

impl Nx {
    pub fn new(
        path: PathBuf,
        action_tx: ActionSender,
        action_rx: ActionReceiver,
    ) -> (Self, mpsc::UnboundedReceiver<String>) {
        let (command_log_tx, command_log_rx) = mpsc::unbounded_channel();

        (
            Self {
                path,
                action_tx,
                action_rx,
                command_log_tx,
            },
            command_log_rx,
        )
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            let Some(event) = self.action_rx.recv().await.ok() else {
                continue;
            };

            let action = match event {
                Action::GetProjects => Action::Projects(self.try_projects().await?),
                _ => continue,
            };
            if self.action_tx.send(action).is_err() {
                break;
            }
        }

        Ok(())
    }

    fn command(&self) -> Command {
        let mut cmd = Command::new("nx");
        cmd.current_dir(self.path.as_path());
        cmd.command_log_tx(self.command_log_tx.clone());

        cmd
    }

    async fn run_command(&self, mut cmd: Command) -> Result<Output> {
        let mut map = StreamMap::new();
        let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;

        if let Some(stdout) = child.stdout.take() {
            let stdout = Box::pin(LinesStream::new(BufReader::new(stdout).lines()))
                as Pin<Box<dyn Stream<Item = io::Result<String>> + Send>>;

            map.insert(IoType::StdOut, stdout);
        }
        if let Some(stderr) = child.stderr.take() {
            let stderr = Box::pin(LinesStream::new(BufReader::new(stderr).lines()))
                as Pin<Box<dyn Stream<Item = io::Result<String>> + Send>>;

            map.insert(IoType::StdErr, stderr);
        }

        let mut stdout = String::new();
        let mut stderr = String::new();

        while let Some((key, val)) = map.next().await {
            let line = val?;

            debug!("[{key}] {line}");

            match key {
                IoType::StdOut => stdout.push_str(&line),
                IoType::StdErr => stderr.push_str(&line),
            }
        }

        let status = child.wait().await?;

        Ok(Output {
            status,
            stdout,
            stderr,
        })
    }

    pub async fn try_projects(&self) -> Result<String> {
        let mut cmd = self.command();
        cmd.arg("show").arg("projects");

        let output = self.run_command(cmd).await?;

        Ok(output.stdout)
    }
}
