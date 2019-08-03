use crate::{Client, Protocol};
use futures::{
    compat::{Future01CompatExt, Stream01CompatExt},
    stream::StreamExt,
};
use log::{debug, info};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use shiplift::rep::ContainerCreateInfo;
use shiplift::{rep::ContainerDetails, ContainerOptions, PullOptions, RmContainerOptions};
use std::collections::HashMap;
use std::net::SocketAddrV4;

struct Port {
    pub source: u32,
    pub host: u32,
    pub protocol: Protocol,
}

/// Abstraction of a running Docker container.
///
/// Use the [new](Container::new)
/// method to create a Container with sensible defaults, or the [builder](Container::builder)
/// method if you need advanced features.
///
/// Container constructors return a future which will be resolved to a Container.
/// The Containers will NOT clean themselves up when they are dropped, you must call the
/// [delete](Container::delete) method on them to remove the container from the host machine.
pub struct Container {
    pub(crate) details: ContainerDetails,

    client: Client,
}

impl Container {
    /// Create a new Docker container.
    ///
    /// # Example
    ///  ```
    /// # #![feature(async_await)]
    /// # use futures::{compat::Stream01CompatExt, stream::StreamExt};
    /// use harbourmaster::Container;
    /// use tokio;
    ///
    ///     let future03 = async {
    ///         let image = "alpine";
    ///
    /// #        // make sure image actually exists locally!
    /// #        use shiplift::{Docker, PullOptions};
    /// #        let mut stream = Docker::default()
    /// #            .images()
    /// #            .pull(
    /// #                &PullOptions::builder()
    /// #                    .image(image.clone())
    /// #                    .tag("latest")
    /// #                    .build(),
    /// #            )
    /// #            .compat();
    /// #
    /// #        while let Some(Ok(status)) = stream.next().await {
    /// #            println!("{}", status);
    /// #        }
    /// #
    /// #        println!("pulled image: {}", &image);
    /// #
    ///         let container = Container::new(image).await.unwrap();
    ///         println!("container created!");
    ///
    ///         container.delete().await.unwrap();
    ///         println!("container deleted!");
    ///
    ///         Ok(())
    ///     };
    ///
    ///     // For the time being, we have to convert the future from a future-0.3 to a future-0.1 to run on the tokio executor
    ///     use futures::future::{FutureExt, TryFutureExt};
    ///     let future01 = future03.boxed().compat();
    ///
    ///     tokio::run(future01);
    ///
    ///  ```
    pub async fn new(image_name: impl Into<String>) -> Result<Container, shiplift::Error> {
        ContainerBuilder::new(image_name).build().await
    }

    /// Create a new Docker container with advanced configuration.
    ///
    /// Check the [ContainerBuilder](ContainerBuilder) documentation for the full
    /// list of options.
    ///
    /// # Example
    /// ```
    /// # #![feature(async_await)]
    /// use harbourmaster::{Container, Protocol};
    /// use tokio;
    ///
    ///     let future03 = async {
    ///         let container = Container::builder("couchdb")
    ///             // the docker image tag to use
    ///             .tag("2.3.0")
    ///
    ///             // set the name of the docker container
    ///             .name("test_container")
    ///
    ///             // optionally add an alphanumeric 'slug' to the
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
    ///         println!("container created!");
    ///
    ///         container.delete().await.unwrap();
    ///         println!("container deleted!");
    ///
    ///         Ok(())
    ///     };
    ///
    ///     // For the time being, we have to convert the future from a future-0.3 to a future-0.1 to run on the tokio executor
    ///     use futures::future::{FutureExt, TryFutureExt};
    ///     let future01 = future03.boxed().compat();
    ///
    ///     tokio::run(future01);
    ///
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
        let map = self
            .details
            .network_settings
            .ports
            .clone()
            .unwrap_or_default();

        unimplemented!()
    }

    /// Exposes the underlying representation of the Docker container's ports. It's messy, this part of
    /// the API will change shortly.
    pub fn ports_raw(&self) -> &Option<HashMap<String, Option<Vec<HashMap<String, String>>>>> {
        &self.details.network_settings.ports
    }

    /// Delete the running docker container.
    ///
    /// This is equivalent to calling `docker rm -f [container]`.
    pub async fn delete(self) -> Result<(), shiplift::Error> {
        self.client
            .containers()
            .get(&self.id())
            .remove(RmContainerOptions::builder().force(true).build())
            .compat()
            .await
    }
}

pub type SourcePort = (u16, Protocol);
pub type HostPort = SocketAddrV4;

/// Builder struct for fine control over the construction of a [Container](Container).
///
/// see [Container::builder()](Container::builder) for example.
pub struct ContainerBuilder {
    image_name: String,
    image_tag: String,
    name: Option<String>,
    ports: Vec<Port>,

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

    /// Use an alternative Docker [Client](Client) to manipulate the Container.ContainerBuilder
    ///
    /// This defaults to a globally shared Docker client at the default socket. This should be
    /// fine in just about all cases.
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
            let slug: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(self.slug_length)
                .collect();

            Some(base_name + "_" + &slug)
        } else {
            Some(base_name)
        }
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

    /// Set whether the client will attempt to pull the image from the internet before
    /// running the container. defaults to false.
    pub fn pull_on_build(mut self, pull: bool) -> Self {
        self.pull_on_build = pull;
        self
    }

    /// Consume the ContainerBuilder and return a future which resolves to the Container (or an error!).
    pub async fn build(self) -> Result<Container, shiplift::Error> {
        let image = self.image();
        let name = self.slugged_name();
        let ports = self.ports;
        let client = self.client;

        if self.pull_on_build {
            pull_image(&client, &image).await?;
        }

        let create_info = create_container(&client, &image, name, ports).await?;
        let id = create_info.id;
        run_container(&client, &id).await?;
        let details = inspect_container(&client, &id).await?;
        Ok(Container { details, client })
    }
}

async fn pull_image(client: &Client, image: &str) -> Result<(), shiplift::Error> {
    info!("pulling image: {}", &image);

    let mut stream = client
        .images()
        .pull(&PullOptions::builder().image(image).build())
        .compat();
    while let Some(Ok(chunk)) = stream.next().await {
        //let chunk = chunk?;
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
) -> Result<ContainerCreateInfo, shiplift::Error> {
    let mut container_options = ContainerOptions::builder(image);

    if let Some(name) = container_name.as_ref() {
        container_options.name(name.as_ref());
    }

    for port in ports {
        container_options.expose(port.source, port.protocol.as_ref(), port.host);
    }

    client
        .containers()
        .create(&container_options.build())
        .compat()
        .await
}

async fn run_container(client: &Client, id: &str) -> Result<(), shiplift::Error> {
    client.containers().get(&id).start().compat().await
}

async fn inspect_container(client: &Client, id: &str) -> Result<ContainerDetails, shiplift::Error> {
    client.containers().get(&id).inspect().compat().await
}
