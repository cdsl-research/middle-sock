use std::{collections::HashMap, io, net::Ipv4Addr, path::Path};

use log::info;
use network::{
    add_address, add_address_with_ns, add_ns, create_veth_pair, set_link_up, set_link_up_with_ns,
    set_veth_to_ns,
};
use process::ProcessExecutor;
use route::{Route, RouteInfo, SEG_1, SEG_2, SEG_3, SEG_4};

mod route;

pub fn new_route<P: AsRef<Path>>(path: P) -> io::Result<Vec<Route>> {
    Route::new(path)
}

pub fn init_routeinfo_map() -> HashMap<String, RouteInfo> {
    HashMap::new()
}

mod network;

pub fn setup_ns<
    T: Into<String> + Clone + std::panic::UnwindSafe,
    U: Into<Ipv4Addr> + Clone + std::panic::UnwindSafe,
>(
    link_name_new: T,
    link_name_host: T,
    ns_name: T,
    ip: U,
    route_info: &RouteInfo,
) -> io::Result<()> {
    let prefix = mask_to_prefix(route_info.mask);
    let prefix_rpos = 32 - prefix;
    let ip_octets: [u8; 4] = ip.clone().into().octets();
    let ip_octet_0 = u64::from(ip_octets[0]);
    let ip_octet_1 = u64::from(ip_octets[1]);
    let ip_octet_2 = u64::from(ip_octets[2]);
    let ip_octet_3 = u64::from(ip_octets[3]);
    let ip_to_u64 = (ip_octet_0 << 24) + (ip_octet_1 << 16) + (ip_octet_2 << 8) + ip_octet_3;
    let ip_range_fixed: u64 = (ip_to_u64 & (0xFFFFFFFF << prefix_rpos)) + 1;
    let peer_ip = Ipv4Addr::new(
        ((ip_range_fixed & SEG_1) >> 24) as u8,
        ((ip_range_fixed & SEG_2) >> 16) as u8,
        ((ip_range_fixed & SEG_3) >> 8) as u8,
        (ip_range_fixed & SEG_4) as u8,
    );
    add_ns(ns_name.clone())?;
    create_veth_pair(link_name_new.clone(), link_name_host.clone())?;
    set_veth_to_ns(link_name_host.clone(), ns_name.clone())?;
    add_address(link_name_new.clone(), peer_ip, prefix)?;
    add_address_with_ns(
        link_name_host.clone(),
        ip.clone().into(),
        prefix,
        ns_name.clone(),
    )?;
    set_link_up(link_name_new.clone())?;
    set_link_up_with_ns(link_name_host.clone(), ns_name.clone())?;
    info!("setup_ns done!");
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

pub fn run_process<T: Into<String> + Clone>(cmd: T, netns_name: T) -> io::Result<()> {
    let mut executor = ProcessExecutor::new(cmd);

    info!("run_process");

    if executor.run(netns_name).is_err() {
        panic!("panic on executor");
    }

    Ok(())
}

pub mod socket;
