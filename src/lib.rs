use std::{collections::HashMap, io, path::Path};

use route::{Route, RouteInfo};

mod route;

pub fn new_route<P: AsRef<Path>>(path: P) -> io::Result<Vec<Route>> {
    Route::new(path)
}

pub fn init_routeinfo_map() -> HashMap<String, RouteInfo> {
    HashMap::new()
}

mod network;



mod packet;
mod process;

async fn run_process<T: Into<String>>(cmd: T) -> io::Result<()> {
    Ok(())
}

pub mod socket;
