#![warn(clippy::all)]
#![warn(missing_docs)]
#![feature(async_await)]

//! [![Latest Docs](https://docs.rs/harbourmaster/badge.svg)](https://docs.rs/harbourmaster/)
//!
//! Harbourmaster is a library of high-level abstractions of Docker objects.
//!
//! Harbourmaster is built on top of the excellent '[shiplift](https://github.com/softprops/shiplift)', but provides an object-oriented interface that
//! is a little easier to work with for some use cases. It also converts shiplift's futures-0.1 to futures-0.3, providing access to the magical and exciting world of async/await syntax
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

pub use shiplift::Error;
