use std::{os::unix::net::UnixStream, path::Path, net::UdpSocket};

// Unix Domain Socket means raw socket (no socket file, probably)

const RECEIVER_PORT: i32 = 67;
const SENDER_PORT: i32 = 68;

#[derive(Debug)]
struct Socket {
    receiver: UdpSocket,
    sender: UdpSocket,
    domain: UnixStream
}

impl Socket {
    pub fn new<P: AsRef<Path>>(fp: P) -> Result<Self, std::io::Error> {
        let receiver_sock = UdpSocket::bind(format!("0.0.0.0:{}", RECEIVER_PORT))?;
        let sender_sock = UdpSocket::bind(format!("0.0.0.0:{}", SENDER_PORT))?;
        let domain_sock = UnixStream::connect(fp)?;
        Ok(Self { receiver: receiver_sock, sender: sender_sock, domain: domain_sock })
    }
}
