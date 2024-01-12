use std::{io, path::Path, env, sync::Arc};

use dhcproto::v4::{CLIENT_PORT, SERVER_PORT};

use log::{info, debug};
use tokio::{net::{UdpSocket, UnixStream}, sync::oneshot};

use crate::packet::DHCPMessage;

#[derive(Debug)]
pub struct Socket {
    receiver: Arc<UdpSocket>,
    sender: Arc<UdpSocket>,
    domain: Option<Arc<UnixStream>>,
}

impl Socket {
    pub async fn new<P: AsRef<Path>>(fp: P) -> io::Result<Self> {
        let receiver_sock = UdpSocket::bind(format!("0.0.0.0:{}", SERVER_PORT)).await?;
        let sender_sock = UdpSocket::bind(format!("0.0.0.0:{}", CLIENT_PORT)).await?;
        let domain_sock = UnixStream::connect(fp).await?;
        Ok(Self {
            receiver: Arc::new(receiver_sock),
            sender: Arc::new(sender_sock),
            domain: Some(Arc::new(domain_sock)),
        })
    }

    pub async fn new_without_domain() -> io::Result<Self> {
        let receiver_sock = UdpSocket::bind(format!("0.0.0.0:{}", SERVER_PORT)).await?;
        let sender_sock = UdpSocket::bind(format!("0.0.0.0:{}", CLIENT_PORT)).await?;
        Ok(Self {
            receiver: Arc::new(receiver_sock),
            sender: Arc::new(sender_sock),
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
        let receiver_sock = Arc::clone(&self.receiver);
        let sender_sock = Arc::clone(&self.sender);
        tokio::spawn(async move {
            info!("spawning receiver");
            // receiver process
        }).await?;
        if let Some(s) = &self.domain {
            let domain_sock = Arc::clone(s);
            tokio::spawn(async move {
                info!("spawning sender (unix domain sock)");
                // sender process w/ unix domain sock
            }).await?;
        } else {
            tokio::spawn(async move {
                info!("spawning sender (udp)");
                // sender process w/ udp
            }).await?;
        }
        Ok(())
    }
}
