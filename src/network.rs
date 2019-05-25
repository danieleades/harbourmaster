use crate::Client;
use shiplift::{builder::NetworkCreateOptionsBuilder, NetworkCreateOptions};
use tokio::prelude::Future;

// Abstraction of a temporary Docker network that cleans up after itself when dropped.
pub struct Network {
    id: String,
    client: Client,
}

impl Network {
    /// Return a Future which resolves to a new Network.
    ///
    /// # Example
    /// ```
    /// use harbourmaster::Network;
    /// use tokio::prelude::Future;
    ///
    /// let fut = Network::new("my cool network")
    ///    .and_then(|net| net.delete())
    ///    .map_err(|e| println!("Error: {}", e));
    ///
    /// tokio::run(fut);
    /// ```
    ///
    pub fn new(name: impl AsRef<str>) -> impl Future<Item = Self, Error = shiplift::Error> {
        NetworkBuilder::new(name).build()
    }

    pub fn builder(name: impl AsRef<str>) -> NetworkBuilder {
        NetworkBuilder::new(name)
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn delete(self) -> impl Future<Item = (), Error = shiplift::Error> {
        self.client.networks().get(&self.id).delete()
    }
}

pub struct NetworkBuilder {
    client: Client,
    options: NetworkCreateOptionsBuilder,
}

impl NetworkBuilder {
    fn new(name: impl AsRef<str>) -> Self {
        NetworkBuilder {
            client: Client::default(),
            options: NetworkCreateOptions::builder(name.as_ref()),
        }
    }

    pub fn build(self) -> impl Future<Item = Network, Error = shiplift::Error> {
        self.client
            .networks()
            .create(&self.options.build())
            .map(|info| Network {
                id: info.id,
                client: self.client,
            })
    }
}
