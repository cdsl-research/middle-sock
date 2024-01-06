use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader},
    net::Ipv4Addr,
    num::ParseIntError,
    path::Path,
};

const RTF_UP: u32 = 0x0001;
const RTF_GATEWAY: u32 = 0x0002;
const SEG_1: u64 = 0xFF000000;
const SEG_2: u64 = 0x00FF0000;
const SEG_3: u64 = 0x0000FF00;
const SEG_4: u64 = 0x000000FF;

// Export from /proc/net/route defines
// ref: https://github.com/torvalds/linux/blob/v6.6/net/ipv4/fib_trie.c#L2976-L3024
#[derive(Debug, Clone)]
struct Route {
    iface: String,    // %s
    destination: u64, // %08X
    gateway: u64,     // %08X
    flags: u32,       // %04X
    ref_cnt: i32,     // %d
    use_field: u32,   // %u
    metric: i32,      // %d
    mask: u64,        // %08X
    mtu: i32,         // %d
    window: u32,      // %u
    irtt: u32,        // %u
}

impl Route {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Vec<Self>> {
        let route = Route::read_route(path.as_ref())?;

        Ok(route)
    }

    fn read_route(path: &Path) -> io::Result<Vec<Self>> {
        let f = File::open(path)?;
        let lines = BufReader::new(f).lines();
        let output: Vec<_> = lines
            .skip(1)
            .filter_map(|v| v.ok())
            .map(|v| v.split("\t").map(str::to_owned).collect())
            .filter_map(|v| Route::vec_to_route(v).ok())
            .collect();
        Ok(output)
    }

    fn vec_to_route(v: Vec<String>) -> Result<Route, ParseIntError> {
        let iface: String = v[0].to_owned();
        let destination: u64 = v[1].parse()?;
        let gateway: u64 = v[2].parse()?;
        let flags: u32 = v[3].parse()?;
        let ref_cnt: i32 = v[4].parse()?;
        let use_field: u32 = v[5].parse()?;
        let metric: i32 = v[6].parse()?;
        let mask: u64 = v[7].parse()?;
        let mtu: i32 = v[8].parse()?;
        let window: u32 = v[9].parse()?;
        let irtt: u32 = v[10].parse()?;
        let r = Route {
            iface,
            destination,
            gateway,
            flags,
            ref_cnt,
            use_field,
            metric,
            mask,
            mtu,
            window,
            irtt,
        };
        Ok(r)
    }

    pub fn parse_network(&self, map: &mut HashMap<String, RouteInfo>) -> io::Result<()> {
        if self.flags & RTF_UP == RTF_UP {
            if self.gateway != 0 && self.flags & RTF_GATEWAY == RTF_GATEWAY {
                let mut route_info = if let Some(v) = map.get(&self.iface) {
                    *v
                } else {
                    let v = RouteInfo::default();
                    map.insert(self.iface.clone(), v);
                    v
                };
                route_info.gateway = Ipv4Addr::new(
                    (self.gateway & SEG_4) as u8,
                    ((self.gateway & SEG_3) >> 8) as u8,
                    ((self.gateway & SEG_2) >> 16) as u8,
                    ((self.gateway & SEG_1) >> 24) as u8,
                );
                map.insert(self.iface.clone(), route_info);
            } else {
                let mut mask = self.mask;
                let mut segment: u8 = 0;
                loop {
                    segment += (mask & 1) as u8;
                    mask >>= 1;
                    if mask == 0 {
                        break;
                    }
                }
                if (self.destination >> segment) == 0 {
                    let mut route_info = if let Some(v) = map.get(&self.iface) {
                        *v
                    } else {
                        let v = RouteInfo::default();
                        map.insert(self.iface.clone(), v);
                        v
                    };
                    route_info.destination = Ipv4Addr::new(
                        (self.destination & SEG_4) as u8,
                        ((self.destination & SEG_3) >> 8) as u8,
                        ((self.destination & SEG_2) >> 16) as u8,
                        ((self.destination & SEG_1) >> 24) as u8,
                    );
                    route_info.mask = Ipv4Addr::new(
                        (self.mask & SEG_4) as u8,
                        ((self.mask & SEG_3) >> 8) as u8,
                        ((self.mask & SEG_2) >> 16) as u8,
                        ((self.mask & SEG_1) >> 24) as u8,
                    );
                    map.insert(self.iface.clone(), route_info);
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct RouteInfo {
    pub destination: Ipv4Addr,
    pub gateway: Ipv4Addr,
    pub mask: Ipv4Addr,
}

impl Default for RouteInfo {
    fn default() -> Self {
        let default_v4 = Ipv4Addr::new(0, 0, 0, 0);
        Self {
            destination: default_v4,
            gateway: default_v4,
            mask: default_v4,
        }
    }
}
