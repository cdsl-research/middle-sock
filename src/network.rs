use std::{
    error,
    fs::File,
    net::{IpAddr, Ipv4Addr},
    os::unix::prelude::AsRawFd, thread, sync::Arc,
};

use futures::TryStreamExt;
use log::{info, debug};
use nix::sched::{setns, CloneFlags};
use rtnetlink::{Error, Handle, NetworkNamespace, NETNS_PATH};
use tokio::runtime::Handle as TokioHandle;

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

pub async fn add_ns<T: Into<String>>(name: T) -> Result<(), Error> {
    NetworkNamespace::add(name.into()).await?;
    Ok(())
}

pub async fn create_veth_pair<T: Into<String> + Clone>(
    link_name_1: T,
    link_name_2: T,
    handle: &Handle,
) -> Result<(), Error> {
    handle
        .link()
        .add()
        .veth(link_name_1.clone().into(), link_name_2.clone().into())
        .execute()
        .await?;
    Ok(())
}

pub async fn add_address<T: Into<String> + Clone, U: Into<IpAddr>>(
    link_name: T,
    ip: U,
    prefix: u8,
    handle: &Handle,
) -> Result<(), Error> {
    let mut links = handle
        .link()
        .get()
        .match_name(link_name.clone().into())
        .execute();
    if let Some(link) = links.try_next().await? {
        handle
            .address()
            .add(link.header.index, ip.into(), prefix)
            .execute()
            .await?
    } else {
        info!("skipped");
    }
    Ok(())
}

pub async fn add_address_with_ns<
    T: Into<String> + Clone,
    U: Into<Ipv4Addr> + Clone,
>(
    link_name: T,
    ip: U,
    prefix: u8,
    handle: &Handle,
    ns_name: T,
) -> Result<(), Box<dyn error::Error>> {
    let new_ns = File::open(format!("{}{}", NETNS_PATH, ns_name.into()))?;
    let handle = Arc::new(handle.clone());
    let h = Arc::clone(&handle);
    let link_name: Arc<String> = Arc::new(link_name.clone().into());
    let l = Arc::clone(&link_name);
    let ip: Arc<Ipv4Addr> = Arc::new(ip.clone().into());
    let i = Arc::clone(&ip);
    let handle = TokioHandle::current();
    let t = thread::spawn(move || {
        handle.block_on(async move {
            if setns(new_ns, CloneFlags::CLONE_NEWNET).is_ok() {
                tokio::spawn(async move {
                    let _ = add_address((*l).clone(),*i, prefix, &h).await;
                });
            }
        })
    });

    t.join().unwrap();

    Ok(())
}

pub async fn set_veth_to_ns<T: Into<String>>(
    link_name: T,
    ns_name: T,
    handle: &Handle,
) -> Result<(), Box<dyn error::Error>> {
    let f = File::open(format!("{}{}", NETNS_PATH, ns_name.into()))?;
    let mut links = handle.link().get().match_name(link_name.into()).execute();
    if let Some(link) = links.try_next().await? {
        handle
            .link()
            .set(link.header.index)
            .setns_by_fd(f.as_raw_fd())
            .execute()
            .await?;
    } else {
        info!("skipped");
    }
    Ok(())
}

pub async fn set_link_up<T: Into<String>>(link_name: T, handle: &Handle) -> Result<(), Error> {
    let mut links = handle.link().get().match_name(link_name.into()).execute();
    if let Some(link) = links.try_next().await? {
        debug!("link (set_link_up) {:?}", link);
        handle.link().set(link.header.index).up().execute().await?
    } else {
        info!("skipped");
    }
    Ok(())
}

pub async fn set_link_up_with_ns<'a, T: Into<String> + Clone + Send + 'a>(
    link_name: T,
    handle: &Handle,
    ns_name: T,
) -> Result<(), Box<dyn error::Error>> {
    let new_ns = File::open(format!("{}{}", NETNS_PATH, ns_name.into()))?;
    let handle = Arc::new(handle.clone());
    let h = Arc::clone(&handle);
    let link_name: Arc<String> = Arc::new(link_name.clone().into());
    let l = Arc::clone(&link_name);
    let handle = TokioHandle::current();
    let t = thread::spawn(move || {
        handle.block_on(async move {
            if setns(new_ns, CloneFlags::CLONE_NEWNET).is_ok() {
                tokio::spawn(async move {
                    debug!("link_name (set_link_up_with): {:?}", (*l).clone());
                    let _ = set_link_up("lo", &h).await;
                    let _ = set_link_up((*l).clone(), &h).await;
                });
            }
        })
    });

    t.join().unwrap();

    Ok(())
}
