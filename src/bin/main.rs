use std::{
    env, error, io,
    net::{IpAddr, SocketAddr},
};

use clap::Parser;
use middle_sock::{init_routeinfo_map, new_route, run_process, setup_ns, socket::Socket};

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short, long, help = "command middle-sock executes")]
    command: String,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    env_logger::init();
    let server_host = env::var("SERVER_HOST")
        .expect("no data in `SERVER_HOST`")
        .parse::<SocketAddr>()
        .expect("could not parse `SERVER_HOST`");

    let cli = Cli::parse();

    let route_file = "/mnt/route";

    let route = new_route(route_file)?;
    let mut route_info = init_routeinfo_map();

    for r in route {
        r.parse_network(&mut route_info)?;
    }

    let ns_name = "dhcp";
    let link_name = "veth0";

    let ip = match server_host.ip() {
        IpAddr::V4(v) => v,
        IpAddr::V6(_) => todo!(),
    };

    for (k, v) in route_info.iter() {
        if !v.is_full() {
            continue;
        }
        setup_ns(link_name, k, ns_name, ip, v)?
    }

    let cmd = cli.command;

    run_process(cmd, ns_name.to_string())?;

    let main_rt = tokio::runtime::Runtime::new()?;
    main_rt.block_on(async {
        let sock = Socket::new_without_domain().await?;
        sock.listen(server_host).await?;
        Ok::<(), io::Error>(())
    })?;
    Ok(())
}
