use std::{
    fs::File,
    io,
    os::{
        fd::AsFd,
        unix::prelude::OwnedFd,
    },
    process::{Command, Stdio},
};

use log::info;
use nix::sched::{setns, CloneFlags};
use rtnetlink::{Error, NetworkNamespace, NETNS_PATH, SELF_NS_PATH};

#[derive(Debug)]
struct ProcessExecutor {
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
        let id = child.id();
        info!("spawned child process; id: {}", id);
        Ok(())
    }
}

pub fn get_current_netns() -> io::Result<OwnedFd> {
    let f = File::open(SELF_NS_PATH)?;
    f.as_fd().try_clone_to_owned()
}

async fn add_ns<T: Into<String>>(name: T) -> Result<(), Error> {
    NetworkNamespace::add(name.into()).await?;
    Ok(())
}

pub fn switch_netns<T: Into<String>>(netns_name: T) -> io::Result<()> {
    let f = File::open(format!("{}/{}", NETNS_PATH, netns_name.into()))?;
    setns(f.as_fd(), CloneFlags::CLONE_NEWNET)?;
    Ok(())
}
