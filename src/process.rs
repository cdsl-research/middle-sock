use std::{
    io,
    process::Stdio,
};

use tokio::process::Command;
use log::{info, warn};

#[derive(Debug)]
pub struct ProcessExecutor {
    command: Command,
}

impl ProcessExecutor {
    pub fn new<T: Into<String>>(cmd: T) -> Self {
        let cmd: String = cmd.into();
        let replaced_string = cmd.replace("'", "");
        let tokens: Vec<_> = replaced_string.split_whitespace().collect();
        let mut builder = Command::new(tokens[0]);
        if let Some(args) = tokens.get(1..) {
            builder.args(args);
        }
        Self { command: builder }
    }

    pub async fn run(&mut self) -> io::Result<()> {
        let child = self
            .command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        if let Some(id) = child.id() {
            info!("spawned child process; id: {}", id);
        } else {
            warn!("The process has been polled to completion.");
        }
        Ok(())
    }
}
