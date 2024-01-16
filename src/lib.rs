use std::{collections::HashMap, error, io, net::Ipv4Addr, os::unix::prelude::AsFd, path::Path};

use network::{add_ns, add_route, create_macvlan_with_address, set_link_up, set_macvlan_to_ns};
use process::{get_current_netns, switch_netns, switch_netns_fd, ProcessExecutor};
use route::{Route, RouteInfo};
use rtnetlink::Handle;

mod route;

pub fn new_route<P: AsRef<Path>>(path: P) -> io::Result<Vec<Route>> {
    Route::new(path)
}

pub fn init_routeinfo_map() -> HashMap<String, RouteInfo> {
    HashMap::new()
}

mod network;

pub async fn setup_ns<T: Into<String> + Clone, U: Into<Ipv4Addr> + Clone>(
    link_name: T,
    new_link_name: T,
    ns_name: T,
    ip: U,
    info: &RouteInfo,
    handle: &Handle,
) -> Result<(), Box<dyn error::Error>> {
    let prefix = mask_to_prefix(info.mask);
    add_ns(ns_name.clone()).await?;
    create_macvlan_with_address(
        link_name.clone(),
        new_link_name.clone(),
        ip.clone().into(),
        prefix,
        handle,
    )
    .await?;
    set_link_up(new_link_name.clone(), handle).await?;
    set_macvlan_to_ns(new_link_name.clone(), ns_name.clone(), handle).await?;
    add_route(ip.clone().into(), prefix, info.gateway, handle).await?;
    Ok(())
}

fn mask_to_prefix(mask: Ipv4Addr) -> u8 {
    let mut prefix: u8 = 0;
    for mut v in mask.octets() {
        while v != 0 {
            prefix += v & 1;
            v >>= 1;
        }
    }
    prefix
}

mod packet;
mod process;

pub async fn run_process<T: Into<String> + Clone>(cmd: T, netns_name: T) -> io::Result<()> {
    let mut executor = ProcessExecutor::new(cmd);
    let current_ns = get_current_netns()?;
    switch_netns(netns_name)?;
    executor.run()?;
    switch_netns_fd(current_ns.as_fd())?;
    Ok(())
}

pub mod socket;
