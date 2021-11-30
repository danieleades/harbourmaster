use std::fmt::Debug;

use crate::Client;
use shiplift::{builder::NetworkCreateOptionsBuilder, NetworkCreateOptions};

/// Abstraction of a Docker network
#[derive(Debug, Clone)]
pub struct Network {
    id: String,
    client: Client,
}

impl Network {
    /// Return a Future which resolves to a new Network.
    pub async fn new(name: impl AsRef<str>) -> Result<Self, shiplift::Error> {
        Builder::new(name).build().await
    }

    /// Create a network using advanced configuration
    pub fn builder(name: impl AsRef<str>) -> Builder {
        Builder::new(name)
    }

    /// The unique id of the Docker network
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Remove the Docker network
    pub async fn delete(self) -> Result<(), shiplift::Error> {
        self.client.networks().get(&self.id).delete().await
    }
}

pub struct Builder {
    client: Client,
    options: NetworkCreateOptionsBuilder,
}

impl std::fmt::Debug for Builder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Builder")
            .field("client", &self.client)
            .field("options", &"NetworkCreateOptionsBuilder")
            .finish()
    }
}

impl Builder {
    fn new(name: impl AsRef<str>) -> Self {
        Self {
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
