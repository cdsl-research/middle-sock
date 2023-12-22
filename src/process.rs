// TODO: add fork/exec code

use std::{io::Result, process::{Command, Stdio}};

use log::info;

#[derive(Debug)]
struct ProcessExecutor {
    command: Command
}

impl ProcessExecutor {
    pub fn new<T: Into<String>>(cmd: T) -> Self {
        let cmd: String = cmd.into();
        let tokens: Vec<_> = cmd.split_whitespace().collect();
        let mut cmd_value = Command::new(tokens[0]);
        for token in tokens.into_iter() {
            cmd_value.arg(token);
        }
        Self {
            command: cmd_value
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let child = self.command.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
        let id = child.id();
        info!("spawned child process; id: {}", id);
        Ok(())
    }
}
