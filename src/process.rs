use std::{
    fs::File,
    io,
    os::unix::prelude::CommandExt,
    process::{Command, Stdio},
};

use log::info;
use nix::sched::{setns, CloneFlags};
use rtnetlink::NETNS_PATH;

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

    pub fn run<T: Into<String>>(&mut self, netns_name: T) -> io::Result<()> {
        let ns_path = format!("{}{}", NETNS_PATH, netns_name.into());

        let child = unsafe {
            self.command
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .pre_exec(move || {
                    let f = File::open(ns_path.clone())?;
                    setns(f, CloneFlags::CLONE_NEWNET)?;
                    Ok(())
                })
                .spawn()
        }?;
        info!("spawned child process; id: {}", child.id());
        Ok(())
    }
}
