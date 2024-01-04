use std::{net::UdpSocket, os::unix::net::UnixStream, path::Path, io};

// Unix Domain Socket means raw socket (no socket file, probably)

const RECEIVER_PORT: i32 = 67;
const SENDER_PORT: i32 = 68;

#[derive(Debug)]
struct Socket {
    receiver: UdpSocket,
    sender: UdpSocket,
    domain: Option<UnixStream>,
}

impl Socket {
    pub fn new<P: AsRef<Path>>(fp: P) -> io::Result<Self> {
        let receiver_sock = UdpSocket::bind(format!("0.0.0.0:{}", RECEIVER_PORT))?;
        let sender_sock = UdpSocket::bind(format!("0.0.0.0:{}", SENDER_PORT))?;
        let domain_sock = UnixStream::connect(fp)?;
        Ok(Self {
            receiver: receiver_sock,
            sender: sender_sock,
            domain: Some(domain_sock),
        })
    }

    pub fn new_without_domain() -> io::Result<Self> {
        let receiver_sock = UdpSocket::bind(format!("0.0.0.0:{}", RECEIVER_PORT))?;
        let sender_sock = UdpSocket::bind(format!("0.0.0.0:{}", SENDER_PORT))?;
        Ok(Self { receiver: receiver_sock, sender: sender_sock, domain: None })
    }
}
