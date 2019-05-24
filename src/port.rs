use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};

pub enum Protocol {
    Tcp,
    Udp,
}

pub struct ContainerPort {
    source_port: u16,
    host_socket: SocketAddrV4,
    protocol: Protocol,
}
