use std::{
    env,
    net::{IpAddr, SocketAddr},
};

use clap::Parser;
use middle_sock::{init_routeinfo_map, new_route, run_process, setup_ns, socket::Socket};
use rtnetlink::new_connection;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short, long, help = "command middle-sock executes")]
    command: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let (connection, handle, _) = new_connection()?;

    tokio::spawn(connection);

    let ns_name = "dhcp";
    let link_name = "eth0";

    let ip = match server_host.ip() {
        IpAddr::V4(v) => v,
        IpAddr::V6(_) => todo!(),
    };

    for (k, v) in route_info.iter() {
        if !v.is_full() {
            continue;
        }
        setup_ns(link_name, k, ns_name, ip, v, &handle).await?
    }

    let cmd = cli.command;

    run_process(cmd, ns_name.to_string()).await?;

    let sock = Socket::new_without_domain().await?;
    sock.listen(server_host).await?;
    Ok(())
}
