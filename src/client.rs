use lazy_static::lazy_static;
use std::ops;
use std::sync::Arc;

/// Docker client
pub struct Client {
    inner_client: Arc<shiplift::Docker>,
}

impl Client {
    /// Construct a new unique Docker Client. Unless you know you need
    /// a unique Client, you should probably `use Client::default()` which
    /// uses a global Docker client internally
    pub fn new() -> Self {
        Self {
            inner_client: Arc::new(shiplift::Docker::new()),
        }
    }
}

impl Default for Client {
    /// Construct a new Docker Client. Internally this client uses a globally
    /// shared client with a connection pool.
    fn default() -> Self {
        global_client()
    }
}

impl ops::Deref for Client {
    type Target = shiplift::Docker;
    fn deref(&self) -> &Self::Target {
        &self.inner_client
    }
}

impl From<shiplift::Docker> for Client {
    /// Create a new Docker Client from a shiplift::Docker object
    ///
    /// # Example
    /// ```
    /// use harbourmaster::Client;
    ///
    /// let client = Client::from(
    ///     shiplift::Docker::new()
    /// );
    /// ```
    fn from(docker: shiplift::Docker) -> Self {
        Client {
            inner_client: Arc::new(docker),
        }
    }
}

impl From<Arc<shiplift::Docker>> for Client {
    fn from(inner_client: Arc<shiplift::Docker>) -> Self {
        Client { inner_client }
    }
}

impl From<&Arc<shiplift::Docker>> for Client {
    fn from(inner_client: &Arc<shiplift::Docker>) -> Self {
        Client {
            inner_client: Arc::clone(inner_client),
        }
    }
}

impl From<&Client> for Client {
    fn from(client: &Client) -> Self {
        Client {
            inner_client: Arc::clone(&client.inner_client),
        }
    }
}

lazy_static! {
    static ref CLIENT: Client = Client::new();
}

fn global_client() -> Client {
    let r: &Client = &CLIENT;
    Client::from(r)
}
