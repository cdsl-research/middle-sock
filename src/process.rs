use std::{
    io,
    process::{Stdio, Command},
};

use log::info;

#[derive(Debug)]
pub struct ProcessExecutor {
    command: Command,
}

impl ProcessExecutor {
    pub fn new<T: Into<String>>(cmd: T) -> Self {
        let cmd: String = cmd.into();
        let tokens: Vec<_> = cmd.split_whitespace().collect();
        let mut builder = Command::new(tokens[0]);
        if let Some(args) = tokens.get(1..) {
            builder.args(args);
        }
        Self { command: builder }
    }

    pub fn run(&mut self) -> io::Result<()> {
        let child = self
            .command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        info!("spawned child process; id: {}", child.id());
        Ok(())
    }
}
