use std::{
    collections::HashMap,
    error,
    fs::File,
    io,
    net::Ipv4Addr,
    path::Path,
    thread::{self},
};

use network::{
    add_address, add_address_with_ns, add_ns, create_veth_pair, set_link_up, set_link_up_with_ns,
    set_veth_to_ns,
};
use nix::sched::{setns, CloneFlags};
use process::ProcessExecutor;
use route::{Route, RouteInfo, SEG_1, SEG_2, SEG_3, SEG_4};
use rtnetlink::{Handle, NETNS_PATH};

mod route;

pub fn new_route<P: AsRef<Path>>(path: P) -> io::Result<Vec<Route>> {
    Route::new(path)
}

pub fn init_routeinfo_map() -> HashMap<String, RouteInfo> {
    HashMap::new()
}

mod network;

pub async fn setup_ns<'a, T: Into<String> + Clone + Send + 'a, U: Into<Ipv4Addr> + Clone + Send>(
    link_name_new: T,
    link_name_host: T,
    ns_name: T,
    ip: U,
    info: &RouteInfo,
    handle: &Handle,
) -> Result<(), Box<dyn error::Error>> {
    let prefix = mask_to_prefix(info.mask);
    let prefix_rpos = 32 - prefix;
    let ip_octets: [u8; 4] = ip.clone().into().octets();
    let ip_octets_3 = ip_octets[3];
    let ip_to_u64: u64 =
        (ip_octets[0] << 24 + ip_octets[1] << 16 + ip_octets[2] << 8 + ip_octets[3]) as u64;
    let ip_range_fixed: u64 = (ip_to_u64 << prefix_rpos) >> prefix_rpos + ip_octets_3;
    let peer_ip = Ipv4Addr::new(
        ((ip_range_fixed & SEG_1) >> 24) as u8,
        ((ip_range_fixed & SEG_2) >> 16) as u8,
        ((ip_range_fixed & SEG_3) >> 8) as u8,
        (ip_range_fixed & SEG_4) as u8,
    );
    add_ns(ns_name.clone()).await?;
    create_veth_pair(link_name_new.clone(), link_name_host.clone(), handle).await?;
    set_veth_to_ns(link_name_host.clone(), ns_name.clone(), handle).await?;
    add_address(link_name_new.clone(), peer_ip, prefix, handle).await?;
    add_address_with_ns(
        link_name_host.clone(),
        ip.clone().into(),
        prefix,
        handle,
        ns_name.clone(),
    )
    .await?;
    set_link_up(link_name_new.clone(), handle).await?;
    set_link_up_with_ns(link_name_host.clone(), handle, ns_name.clone()).await?;
    // add_route(ip.clone().into(), prefix, info.gateway, handle).await?;
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
    let new_ns = File::open(format!("{}{}", NETNS_PATH, netns_name.into()))?;

    thread::spawn(move || {
        if setns(new_ns, CloneFlags::CLONE_NEWNET).is_ok() {
            if executor.run().is_ok() {
                loop {}
            } else {
                panic!("panic on executor");
            };
        } else {
            panic!("panic on setns");
        }
    });
    Ok(())
}

pub mod socket;
