use crate::{Client, Protocol};
use futures_util::stream::StreamExt;
use log::{debug, info};
use rand::{
    distributions::{Alphanumeric, Distribution},
    thread_rng,
};
use shiplift::{
    rep::{ContainerCreateInfo, ContainerDetails},
    ContainerOptions, PullOptions, RmContainerOptions,
};
use std::{collections::HashMap, net::SocketAddrV4};

struct Port {
    pub source: u32,
    pub host: u32,
    pub protocol: Protocol,
}

/// Abstraction of a running Docker container.
///
/// Use the [new](Container::new)
/// method to create a Container with sensible defaults, or the
/// [builder](Container::builder) method if you need advanced features.
///
/// Container constructors return a future which will be resolved to a
/// Container. The Containers will NOT clean themselves up when they are
/// dropped, you must call the [delete](Container::delete) method on them to
/// remove the container from the host machine.
pub struct Container {
    pub(crate) details: ContainerDetails,

    client: Client,
}

impl Container {
    /// Create a new Docker container.
    ///
    /// # Example
    ///  ```no_run
    /// use harbourmaster::Container;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let container = Container::new("alpine").await.unwrap();
    ///
    ///     // clean up container
    ///     container.delete().await.unwrap();
    /// }
    ///  ```
    pub async fn new(image_name: impl Into<String>) -> Result<Container, shiplift::Error> {
        ContainerBuilder::new(image_name).build().await
    }

    /// Pull an image and create a new Docker container from it.
    ///
    /// This method is identical to `Container::new` except that it will attempt
    /// to pull the image first.
    ///
    /// # Example
    ///  ```no_run
    /// use harbourmaster::Container;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let container = Container::pull("alpine").await.unwrap();
    ///
    ///     // clean up container
    ///     container.delete().await.unwrap();
    /// }
    ///  ```
    pub async fn pull(image_name: impl Into<String>) -> Result<Container, shiplift::Error> {
        ContainerBuilder::new(image_name)
            .pull_on_build(true)
            .build()
            .await
    }

    /// Create a new Docker container with advanced configuration.
    ///
    /// Check the [ContainerBuilder](ContainerBuilder) documentation for the
    /// full list of options.
    ///
    /// # Example
    /// ```no_run
    /// use harbourmaster::{Container, Protocol};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let container = Container::builder("couchdb")
    ///             // the docker image tag to use
    ///             .tag("2.3.0")
    ///     
    ///             // set the name of the docker container
    ///             .name("test_container")
    ///     
    ///             // optionally add a randomised alphanumeric 'slug' to the
    ///             // container name. Useful if you're creating and
    ///             // naming them in bulk
    ///             .slug_length(6)
    ///     
    ///             // expose ports on the container to the host machine
    ///             .expose(5984, 5984, Protocol::Tcp)
    ///     
    ///             // if true, pull the image from the webular information
    ///             // super-highway before building.
    ///             .pull_on_build(true)
    ///     
    ///             // build the container using the above parameters
    ///             .build()
    ///             .await
    ///             .unwrap();
    ///
    ///     println!("container created!");
    ///
    ///     container.delete().await.unwrap();
    ///     println!("container deleted!");
    /// }
    /// ```
    pub fn builder(image_name: impl Into<String>) -> ContainerBuilder {
        ContainerBuilder::new(image_name)
    }

    /// Return the Docker id of the running container
    pub fn id(&self) -> &str {
        &self.details.id
    }

    /// Not yet implemented
    pub fn ports(&self) -> &HashMap<SourcePort, Vec<HostPort>> {
        let _map = self
            .details
            .network_settings
            .ports
            .clone()
            .unwrap_or_default();

        todo!()
    }

    /// Exposes the underlying representation of the Docker container's ports.
    /// It's messy, this part of the API will change shortly.
    pub fn ports_raw(&self) -> &Option<HashMap<String, Option<Vec<HashMap<String, String>>>>> {
        &self.details.network_settings.ports
    }

    /// Delete the running docker container.
    ///
    /// This is equivalent to calling `docker rm -f [container]`.
    pub async fn delete(self) -> Result<(), shiplift::Error> {
        self.client
            .containers()
            .get(self.id())
            .remove(RmContainerOptions::builder().force(true).build())
            .await
    }
}

pub type SourcePort = (u16, Protocol);
pub type HostPort = SocketAddrV4;

/// Builder struct for fine control over the construction of a
/// [Container](Container).
///
/// see [Container::builder()](Container::builder) for example.
pub struct ContainerBuilder {
    image_name: String,
    image_tag: String,
    name: Option<String>,
    ports: Vec<Port>,
    commands: Vec<String>,
    environment_variables: Vec<String>,

    client: Client,

    pull_on_build: bool,
    slug_length: usize,
}

impl ContainerBuilder {
    fn new(image_name: impl Into<String>) -> Self {
        ContainerBuilder {
            image_name: image_name.into(),
            image_tag: String::from("latest"),
            name: None,
            ports: Vec::new(),
            commands: Vec::new(),
            environment_variables: Vec::new(),

            client: Client::default(),

            pull_on_build: false,
            slug_length: 0,
        }
    }

    fn image(&self) -> String {
        format!("{}:{}", self.image_name, self.image_tag)
    }

    /// Set the tag of the docker image. defaults to "latest".
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.image_tag = tag.into();
        self
    }

    /// Use an alternative Docker [Client](Client) to manipulate the
    /// Container.ContainerBuilder
    ///
    /// This defaults to a globally shared Docker client at the default socket.
    /// This should be fine in just about all cases.
    pub fn client(mut self, client: impl Into<Client>) -> Self {
        self.client = client.into();
        self
    }

    /// Set the name of the docker container.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Optionally add an alphanumeric 'slug' to the container name.
    ///
    /// In the form "[container name]_XXXX" where XXXX represents the slug.
    ///
    /// Useful if you're creating a job lot of containers and you want them to
    /// have human readable names, but no collisions.
    pub fn slug_length(mut self, length: usize) -> Self {
        self.slug_length = length;
        self
    }

    fn slugged_name(&self) -> Option<String> {
        let base_name = self.name.clone()?;

        if self.slug_length > 0 {
            let mut rng = thread_rng();

            let slug: String = Alphanumeric
                .sample_iter(&mut rng)
                .take(self.slug_length)
                .map(char::from)
                .collect();

            Some(base_name + "_" + &slug)
        } else {
            Some(base_name)
        }
    }

    /// Run commands when starting the container
    pub fn commands<I, S>(mut self, commands: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.commands = commands.into_iter().map(Into::into).collect();
        self
    }

    /// Expose a port from the container to the host.
    ///
    /// Can be called multiple times to expose multiple ports.
    pub fn expose(mut self, src_port: u16, host_port: u16, protocol: Protocol) -> Self {
        self.ports.push(Port {
            source: src_port.into(),
            host: host_port.into(),
            protocol,
        });
        self
    }

    /// Add an environment variable to the container.
    pub fn environment_variable(mut self, env: impl Into<String>) -> Self {
        self.environment_variables.push(env.into());
        self
    }

    /// Set whether the client will attempt to pull the image from the internet
    /// before running the container. defaults to false.
    pub fn pull_on_build(mut self, pull: bool) -> Self {
        self.pull_on_build = pull;
        self
    }

    /// Consume the ContainerBuilder and return a future which resolves to the
    /// Container (or an error!).
    pub async fn build(self) -> Result<Container, shiplift::Error> {
        let image = self.image();
        let commands = self.commands.iter().map(AsRef::as_ref).collect();

        if self.pull_on_build {
            pull_image(&self.client, &image).await?;
        }

        let create_info = create_container(
            &self.client,
            &image,
            self.slugged_name(),
            self.ports,
            commands,
            self.environment_variables,
        )
        .await?;
        let id = create_info.id;
        run_container(&self.client, &id).await?;
        let details = inspect_container(&self.client, &id).await?;
        Ok(Container {
            details,
            client: self.client,
        })
    }
}

async fn pull_image(client: &Client, image: &str) -> Result<(), shiplift::Error> {
    info!("pulling image: {}", &image);

    let mut stream = client
        .images()
        .pull(&PullOptions::builder().image(image).build());
    while let Some(Ok(chunk)) = stream.next().await {
        // let chunk = chunk?;
        debug!("{}", chunk);
    }

    info!("pulled image: {}", &image);
    Ok(())
}

async fn create_container<S: AsRef<str>>(
    client: &Client,
    image: &str,
    container_name: Option<S>,
    ports: impl IntoIterator<Item = Port>,
    commands: Vec<&str>,
    environment_variables: Vec<String>,
) -> Result<ContainerCreateInfo, shiplift::Error> {
    let mut container_options = ContainerOptions::builder(image);
    container_options.cmd(commands);
    container_options.env(environment_variables);

    if let Some(name) = container_name.as_ref() {
        container_options.name(name.as_ref());
    }

    for port in ports {
        container_options.expose(port.source, port.protocol.as_ref(), port.host);
    }

    client.containers().create(&container_options.build()).await
}

async fn run_container(client: &Client, id: &str) -> Result<(), shiplift::Error> {
    client.containers().get(id).start().await
}

async fn inspect_container(client: &Client, id: &str) -> Result<ContainerDetails, shiplift::Error> {
    client.containers().get(id).inspect().await
}
