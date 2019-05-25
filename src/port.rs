use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};

pub enum Protocol {
    Tcp,
    Udp,
}

impl AsRef<str> for Protocol {
    fn as_ref(&self) -> &str {
        match self {
            Protocol::Tcp => "tcp",
            Protocol::Udp => "udp",
        }
    }
}

pub struct ContainerPort {
    source_port: u16,
    host_socket: SocketAddrV4,
    protocol: Protocol,
}
