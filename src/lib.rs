mod client;
pub use client::Client;
mod container;
mod port;
pub use container::Container;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
