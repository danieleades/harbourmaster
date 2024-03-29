/// Enum representing a port's communication protocol
#[derive(Debug, Clone, Copy)]
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
