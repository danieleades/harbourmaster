#![warn(clippy::all)]
#![warn(missing_docs)]

//! [![Latest Docs](https://docs.rs/harbourmaster/badge.svg)](https://docs.rs/harbourmaster/0.0.0/harbourmaster/)
//! 
//! Harbourmaster is a library of high-level abstractions of Docker objects.
//!
//! Harbourmaster is built on top of the excellent '[shiplift](https://github.com/softprops/shiplift)', but provides an object-oriented interface that
//! is a little easier to work with for some use cases.
//!
//! Particularly useful for unit testing that involves spinning up and then removing Docker containers.
//! 
//! # Usage
//!```
//! use tokio::prelude::Future;
//! use harbourmaster::Container;
//! 
//! let image = "alpine";
//! # // make sure the image actually exists locally!
//! # use tokio::prelude::Stream;
//! # use shiplift::{Docker, PullOptions};
//! # tokio::run(
//! #    Docker::default().images().pull(
//! #          &PullOptions::builder().image(image.clone()).tag("latest").build()
//! #    )
//! #    .for_each(|output| {
//! #       println!("{:?}", output);
//! #       Ok(())
//! #     })
//! #    .map(move |_| {
//! #       println!("pulled image: {}", &image);
//! #       ()
//! #     })
//! #    .map_err( |e| println!("Error: {}", e))
//! # );
//! 
//! let fut = Container::new(image)
//!     .map(
//!         |container| {
//!         println!("container created!");
//!         container
//!     })
//!     .and_then(
//!         |container| {
//!         println!("removing container");
//!         container.delete()
//!     })
//!     .map_err(
//!         |e| println!("Error: {}", e)
//!     );
//! 
//! tokio::run(fut);
//! ```
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
