use std::{
    error,
    fs::File,
    net::{IpAddr, Ipv4Addr},
    os::unix::prelude::AsRawFd,
};

use futures::TryStreamExt;
use log::info;
use rtnetlink::{Error, Handle, NetworkNamespace, NETNS_PATH};

pub async fn add_route<T: Into<Ipv4Addr>>(
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

pub async fn create_macvlan_with_address<T: Into<String> + Clone, U: Into<IpAddr>>(
    link_name: T,
    new_link_name: T,
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
        let request = handle.link().add().macvlan(
            new_link_name.clone().into(),
            link.header.index,
            4u32, // bridge mode
        );
        request.execute().await?;
    } else {
        info!(
            "skipped `create_macvlan` due to no {:?}",
            link_name.clone().into()
        )
    }
    let mut links = handle
        .link()
        .get()
        .match_name(new_link_name.clone().into())
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

pub async fn set_macvlan_to_ns<T: Into<String>>(
    link_name: T,
    ns_name: T,
    handle: &Handle,
) -> Result<(), Box<dyn error::Error>> {
    let f = File::open(format!("{}/{}", NETNS_PATH, ns_name.into()))?;
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
        handle.link().set(link.header.index).up().execute().await?
    } else {
        info!("skipped");
    }
    Ok(())
}
