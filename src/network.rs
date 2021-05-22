use crate::Client;
use shiplift::{builder::NetworkCreateOptionsBuilder, NetworkCreateOptions};

/// Abstraction of a temporary Docker network that cleans up after itself when
/// dropped.
pub struct Network {
    id: String,
    client: Client,
}

impl Network {
    /// Return a Future which resolves to a new Network.
    pub async fn new(name: impl AsRef<str>) -> Result<Self, shiplift::Error> {
        NetworkBuilder::new(name).build().await
    }

    /// Create a network using advanced configuration
    pub fn builder(name: impl AsRef<str>) -> NetworkBuilder {
        NetworkBuilder::new(name)
    }

    /// The unique id of the Docker network
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Remove the Docker network
    pub async fn delete(self) -> Result<(), shiplift::Error> {
        self.client.networks().get(&self.id).delete().await
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
        let create_info = self.client.networks().create(&self.options.build()).await?;

        Ok(Network {
            id: create_info.id,
            client: self.client,
        })
    }
}
