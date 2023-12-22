use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    num::ParseIntError,
    path::Path,
};

// read /proc/route and parse code
// TODO: Add algorithm

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
        let values: Vec<Vec<_>> = lines
            .skip(1)
            .filter_map(|v| v.ok())
            .map(|v| v.split("\t").map(str::to_owned).collect())
            .collect();
        let output: Vec<_> = values
            .into_iter()
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
}
