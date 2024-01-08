use std::{io, path::Path};

use dhcproto::v4::{CLIENT_PORT, SERVER_PORT};
use tokio::net::{UdpSocket, UnixStream};

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
}
