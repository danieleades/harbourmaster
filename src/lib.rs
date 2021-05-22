#![warn(clippy::all)]
#![warn(missing_docs)]

//! [![Build Status](https://travis-ci.org/danieleades/harbourmaster.svg?branch=master)](https://travis-ci.org/danieleades/harbourmaster)
//! [![Latest Docs](https://docs.rs/harbourmaster/badge.svg)](https://docs.rs/harbourmaster/)
//!
//! Harbourmaster is a library of high-level abstractions of Docker objects.
//!
//! Harbourmaster is built on top of the excellent '[shiplift](https://github.com/softprops/shiplift)', but provides an object-oriented interface that
//! is a little easier to work with for some use cases. It's also using
//! async/await-ready futures-0.3 for the interface.
//!
//! Particularly useful for unit testing that involves spinning up and then
//! removing Docker containers.
//!
//! ## Usage
//! ```rust, no_run
//! use harbourmaster::Container;
//!
//! #[tokio::main]
//! async fn main() {
//!
//!     let image = "alpine";
//!
//!     println!("creating container!");
//!     let container = Container::new(image).await.unwrap();
//!     println!("container created!");
//!
//!     println!("removing container!");
//!     container.delete().await.unwrap();
//!     println!("container removed!");
//! }
//! ```

mod client;
pub use client::Client;
mod container;
mod network;
pub use crate::network::Network;
mod port;
pub use crate::port::Protocol;
pub use container::{Container, ContainerBuilder};

pub use shiplift::Error;
