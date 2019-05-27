#![warn(clippy::all)]
#![warn(missing_docs)]

//! Harbourmaster is a library of high-level abstractions of Docker objects.
//!
//! Harbourmaster is built on top of the excellent '[shiplift](https://github.com/softprops/shiplift)', but provides an object-oriented interface that
//! is a little easier to work with for some use cases.
//!
//! Particularly useful for unit testing that involves spinning up and then removing Docker containers.
//!

mod client;
pub use client::Client;
mod container;
mod network;
pub use crate::network::Network;
mod port;
pub use crate::port::Protocol;
pub use container::{Container, ContainerBuilder};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
