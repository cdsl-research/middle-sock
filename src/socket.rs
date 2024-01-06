use std::{io, net::UdpSocket, os::unix::net::UnixStream, path::Path};

use dhcproto::v4::{SERVER_PORT, CLIENT_PORT};

#[derive(Debug)]
struct Socket {
    receiver: UdpSocket,
    sender: UdpSocket,
    domain: Option<UnixStream>,
}

impl Socket {
    pub fn new<P: AsRef<Path>>(fp: P) -> io::Result<Self> {
        let receiver_sock = UdpSocket::bind(format!("0.0.0.0:{}", SERVER_PORT))?;
        let sender_sock = UdpSocket::bind(format!("0.0.0.0:{}", CLIENT_PORT))?;
        let domain_sock = UnixStream::connect(fp)?;
        Ok(Self {
            receiver: receiver_sock,
            sender: sender_sock,
            domain: Some(domain_sock),
        })
    }

    pub fn new_without_domain() -> io::Result<Self> {
        let receiver_sock = UdpSocket::bind(format!("0.0.0.0:{}", SERVER_PORT))?;
        let sender_sock = UdpSocket::bind(format!("0.0.0.0:{}", CLIENT_PORT))?;
        Ok(Self {
            receiver: receiver_sock,
            sender: sender_sock,
            domain: None,
        })
    }
}
