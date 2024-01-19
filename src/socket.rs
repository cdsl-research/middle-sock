use std::{io, net::SocketAddr, path::Path, sync::Arc};

use dhcproto::{
    v4::{Message, CLIENT_PORT, SERVER_PORT},
    Decodable, Decoder, Encodable, Encoder,
};

use log::{debug, info, warn};
use tokio::{
    net::{UdpSocket, UnixStream},
    sync::mpsc,
};

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

    pub async fn listen(&self, server_host: SocketAddr) -> io::Result<()> {
        let runtime_ip = String::from("172.17.0.1");
        let (tx, mut rx) = mpsc::channel::<(DHCPMessage, SocketAddr)>(1024);
        debug!("server_host: {}", &server_host);
        let receiver_sock = Arc::clone(&self.receiver);
        let sender_sock = Arc::clone(&self.sender);
        if let Some(s) = &self.domain {
            let _domain_sock = Arc::clone(s);
            tokio::spawn(async move {
                info!("spawning sender (unix domain sock)");
                // sender process w/ unix domain sock
            });
        } else {
            tokio::spawn(async move {
                info!("spawning sender (udp)");
                // sender process w/ udp
                while let Some((msg, addr)) = rx.recv().await {
                    debug!("(sender task) msg: {:?}, addr: {:?}", msg, addr);
                    if addr.ip().to_string() == runtime_ip {
                        let mut buf = Vec::new();
                        let mut e = Encoder::new(&mut buf);
                        let _ = msg.raw().encode(&mut e);
                        info!("send to host...");
                        if let Err(_) = sender_sock.send_to(&buf, server_host).await {
                            warn!("could not send to server_host")
                        }
                    } else {
                        info!("addr is not from runtime?");
                    }
                }
            });
        }
        info!("spawning receiver");
        let mut buf = [0; 1024];
        loop {
            let (len, addr) = receiver_sock.recv_from(&mut buf).await?;
            let msg = Message::decode(&mut Decoder::new(&buf[..len]));
            if let Ok(msg) = msg {
                info!("DHCP Message received!");
                debug!("msg: {:?}", msg);
                if let Err(_) = tx.send((msg.into(), addr)).await {
                    warn!("failed sending");
                }
            } else {
                warn!("failed decode msg")
            }
        }
    }
}
