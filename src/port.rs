//use std::net::{SocketAddrV4};

/// Enum representing a port's communication protocol
pub enum Protocol {
    /// TCP protocol
    Tcp,

    /// UDP protocol
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

// /// This struct represents 
// struct ContainerPort {
//     source_port: u16,
//     host_socket: SocketAddrV4,
//     protocol: Protocol,
// }
