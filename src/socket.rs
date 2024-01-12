use std::{io, path::Path, env};

use dhcproto::v4::{CLIENT_PORT, SERVER_PORT};

use log::{info, debug};
use tokio::{net::{UdpSocket, UnixStream}, sync::oneshot};

use crate::packet::DHCPMessage;

#[derive(Debug)]
pub struct Socket {
    receiver: UdpSocket,
    sender: UdpSocket,
    domain: Option<UnixStream>,
}

impl Socket {
    pub async fn new<P: AsRef<Path>>(fp: P) -> io::Result<Self> {
        let receiver_sock = UdpSocket::bind(format!("0.0.0.0:{}", SERVER_PORT)).await?;
        let sender_sock = UdpSocket::bind(format!("0.0.0.0:{}", CLIENT_PORT)).await?;
        let domain_sock = UnixStream::connect(fp).await?;
        Ok(Self {
            receiver: receiver_sock,
            sender: sender_sock,
            domain: Some(domain_sock),
        })
    }

    pub async fn new_without_domain() -> io::Result<Self> {
        let receiver_sock = UdpSocket::bind(format!("0.0.0.0:{}", SERVER_PORT)).await?;
        let sender_sock = UdpSocket::bind(format!("0.0.0.0:{}", CLIENT_PORT)).await?;
        Ok(Self {
            receiver: receiver_sock,
            sender: sender_sock,
            domain: None,
        })
    }

    pub async fn listen(&self) -> io::Result<()> {
        let server_host = env::var("SERVER_HOST").expect("no data in `SERVER_HOST`");
        debug!("server_host: {}", &server_host);
        // receiver to sender channel
        let (tx1, rx1) = oneshot::channel::<DHCPMessage>();
        // sender to receiver channel
        let (tx2, rx2) = oneshot::channel::<DHCPMessage>();
        tokio::spawn(async move {
            info!("spawning receiver")
            // receiver process
        }).await?;
        tokio::spawn(async move {
            info!("spawning sender")
            // sender process
        }).await?;
        Ok(())
    }
}
