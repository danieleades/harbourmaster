use crate::Client;
use futures::compat::Future01CompatExt;
use shiplift::{builder::NetworkCreateOptionsBuilder, NetworkCreateOptions};

/// Abstraction of a temporary Docker network that cleans up after itself when dropped.
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
    pub async fn new(name: impl AsRef<str>) -> Result<Self, shiplift::Error> {
        NetworkBuilder::new(name).build().await
    }

    pub fn builder(name: impl AsRef<str>) -> NetworkBuilder {
        NetworkBuilder::new(name)
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub async fn delete(self) -> Result<(), shiplift::Error> {
        self.client.networks().get(&self.id).delete().compat().await
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

    pub async fn build(self) -> Result<Network, shiplift::Error> {
        let create_info = self
            .client
            .networks()
            .create(&self.options.build())
            .compat()
            .await?;

        Ok(Network {
            id: create_info.id,
            client: self.client,
        })
    }
}
