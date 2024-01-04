use std::net::{Ipv4Addr, IpAddr};

use futures::TryStreamExt;
use log::info;
use rtnetlink::{NetworkNamespace, Error, Handle};

async fn add_ns<T: Into<String>>(name: T) -> Result<(), Error> {
    NetworkNamespace::add(name.into()).await?;
    Ok(())
}

async fn add_route<T: Into<Ipv4Addr>>(dest: T, prefix: u8, gateway: T, handle: &Handle) -> Result<(), Error> {
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

async fn create_macvlan<T: Into<String>, U: Into<IpAddr>>(link_name: T, name: T,
    handle: &Handle,
) -> Result<(), Error> {
    let link_name: String = link_name.into();
    let mut links = handle
    .link()
    .get()
    .match_name(link_name.clone())
    .execute();
    if let Some(link) = links.try_next().await? {
        let request = handle.link().add().macvlan(
            name.into(),
            link.header.index,
            4u32, // bridge mode
        );
        request.execute().await?
    } else {
        info!("skipped `create_macvlan` due to no {:?}", &link_name)
    }
    Ok(())
}
