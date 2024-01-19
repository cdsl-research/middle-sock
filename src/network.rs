use std::{
    fs::File,
    io,
    net::{IpAddr, Ipv4Addr},
    os::unix::prelude::AsRawFd,
    process::exit,
};

use futures::TryStreamExt;
use log::{debug, info};
use nix::{
    sched::{setns, CloneFlags},
    sys::wait::waitpid,
    unistd::{fork, ForkResult},
};
use rtnetlink::{new_connection, Error, Handle, NetworkNamespace, NETNS_PATH};

pub async fn _add_route<T: Into<Ipv4Addr>>(
    dest: T,
    prefix: u8,
    gateway: T,
    handle: &Handle,
) -> Result<(), Error> {
    let route = handle.route();
    route
        .add()
        .v4()
        .destination_prefix(dest.into(), prefix)
        .gateway(gateway.into())
        .execute()
        .await?;
    Ok(())
}

pub fn add_ns<T: Into<String>>(name: T) -> io::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let (connection, _, _) = new_connection()?;
        tokio::spawn(connection);
        if let Err(e) = NetworkNamespace::add(name.into()).await {
            Err(io::Error::new(io::ErrorKind::Other, e))
        } else {
            Ok::<(), io::Error>(())
        }
    })?;
    Ok(())
}

pub fn create_veth_pair<T: Into<String> + Clone>(link_name_1: T, link_name_2: T) -> io::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let (connection, handle, _) = new_connection()?;
        tokio::spawn(connection);
        if let Err(e) = handle
            .link()
            .add()
            .veth(link_name_1.clone().into(), link_name_2.clone().into())
            .execute()
            .await
        {
            Err(io::Error::new(io::ErrorKind::Other, e))
        } else {
            Ok::<(), io::Error>(())
        }
    })?;
    Ok(())
}

pub fn add_address<
    T: Into<String> + Clone + std::panic::UnwindSafe,
    U: Into<IpAddr> + Clone + std::panic::UnwindSafe,
>(
    link_name: T,
    ip: U,
    prefix: u8,
) -> io::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let (connection, handle, _) = new_connection()?;
        tokio::spawn(connection);
        let mut links = handle
            .link()
            .get()
            .match_name(link_name.clone().into())
            .execute();
        if let Some(Some(link)) = links.try_next().await.ok() {
            debug!("link (add_address): {:?}", link);
            if let Err(e) = handle
                .address()
                .add(link.header.index, ip.into(), prefix)
                .execute()
                .await
            {
                return Err(io::Error::new(io::ErrorKind::Other, e));
            }
        }
        Ok::<(), io::Error>(())
    })?;

    Ok(())
}

pub fn add_address_with_ns<
    T: Into<String> + Clone + std::panic::UnwindSafe,
    U: Into<Ipv4Addr> + Clone + std::panic::UnwindSafe,
>(
    link_name: T,
    ip: U,
    prefix: u8,
    ns_name: T,
) -> io::Result<()> {
    let ns_path = format!("{}{}", NETNS_PATH, ns_name.into());
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            waitpid(child, None).unwrap();
        }
        Ok(ForkResult::Child) => {
            info!("(add_address_with_ns) fork child!");
            let res = std::panic::catch_unwind(|| -> io::Result<()> {
                let f = File::open(ns_path)?;
                setns(f, CloneFlags::CLONE_NEWNET)?;
                Ok(())
            });
            info!("(add_address_with_ns) changed ns!");
            match res {
                Err(_panic) => {
                    log::error!("child process crashed");
                    std::process::abort()
                }
                Ok(Err(fail)) => {
                    log::error!("child process failed: {}", fail);
                    exit(1)
                }
                Ok(Ok(())) => {
                    info!("(add_address_with_ns) calling add_address");
                    add_address(link_name, ip.into(), prefix)?;
                    info!("(add_address_with_ns) child will exit 0");
                    exit(0);
                }
            }
        }
        Err(_) => {
            panic!("fork() failed");
        }
    }

    Ok(())
}

pub fn set_veth_to_ns<T: Into<String>>(link_name: T, ns_name: T) -> io::Result<()> {
    let f = File::open(format!("{}{}", NETNS_PATH, ns_name.into()))?;
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let (connection, handle, _) = new_connection()?;
        tokio::spawn(connection);
        let mut links = handle.link().get().match_name(link_name.into()).execute();
        if let Some(Some(link)) = links.try_next().await.ok() {
            if let Err(e) = handle
                .link()
                .set(link.header.index)
                .setns_by_fd(f.as_raw_fd())
                .execute()
                .await
            {
                return Err(io::Error::new(io::ErrorKind::Other, e));
            }
        } else {
            info!("skipped");
        }
        Ok::<(), io::Error>(())
    })?;
    Ok(())
}

pub fn set_link_up<T: Into<String> + std::panic::UnwindSafe>(link_name: T) -> io::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let (connection, handle, _) = new_connection()?;
        tokio::spawn(connection);
        let mut links = handle.link().get().match_name(link_name.into()).execute();
        if let Some(Some(link)) = links.try_next().await.ok() {
            debug!("link (set_link_up) {:?}", link);
            if let Err(e) = handle.link().set(link.header.index).up().execute().await {
                return Err(io::Error::new(io::ErrorKind::Other, e));
            }
        }
        Ok::<(), io::Error>(())
    })?;
    Ok(())
}

pub fn set_link_up_with_ns<T: Into<String> + Clone + std::panic::UnwindSafe>(
    link_name: T,
    ns_name: T,
) -> io::Result<()> {
    let ns_path = format!("{}{}", NETNS_PATH, ns_name.into());
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            waitpid(child, None).unwrap();
        }
        Ok(ForkResult::Child) => {
            info!("(set_link_up_with_ns) fork child!");
            let res = std::panic::catch_unwind(|| -> io::Result<()> {
                let f = File::open(ns_path)?;
                setns(f, CloneFlags::CLONE_NEWNET)?;
                Ok(())
            });
            match res {
                Err(_panic) => {
                    log::error!("child process crashed");
                    std::process::abort()
                }
                Ok(Err(fail)) => {
                    log::error!("child process failed: {}", fail);
                    exit(1)
                }
                Ok(Ok(())) => {
                    set_link_up("lo")?;
                    set_link_up(link_name)?;
                    exit(0);
                }
            }
        }
        Err(_) => {
            panic!("fork() failed");
        }
    }

    Ok(())
}
