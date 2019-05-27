use crate::{Client, Protocol};
use log::{debug, error, info, warn};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use shiplift::{rep::ContainerDetails, ContainerOptions, PullOptions, RmContainerOptions};
use std::collections::HashMap;
use std::net::SocketAddrV4;
use tokio::prelude::{future, Future, Stream};

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
    /// ```
    /// use tokio::prelude::Future;
    /// use harbourmaster::Container;
    /// 
    /// let fut = Container::new("alpine")
    ///     .map(
    ///         |container| {
    ///         println!("container created!");
    ///         container
    ///     })
    ///     .and_then(
    ///         |container| {
    ///         println!("removing container");
    ///         container.delete()
    ///     })
    ///     .map_err(
    ///         |e| println!("Error: {}", e)
    ///     );
    /// 
    /// tokio::run(fut);
    /// ```
    pub fn new(image_name: impl Into<String>) -> impl Future<Item = Self, Error = shiplift::Error> {
        ContainerBuilder::new(image_name).build()
    }

    /// Create a new Docker container with advanced configuration.
    /// 
    /// Check the [ContainerBuilder](ContainerBuilder) documentation for the full
    /// list of options.
    /// 
    /// # Example
    /// ```
    /// use tokio::prelude::Future;
    /// use harbourmaster::Container;
    /// 
    /// let fut = Container::builder("couchdb")
    ///     // the docker image tag to use
    ///     .tag("2.3.0")
    /// 
    ///     // set the name of the docker container
    ///     .name("test_container")
    /// 
    ///     // optionally add an alphanumeric 'slug' to the
    ///     // container name. Useful if you're creating and
    ///     // naming them in bulk.
    ///     .slug_length(6)
    /// 
    ///     // if true, pull the image from the webular information
    ///     // super-highway before building.
    ///     .pull_on_build(true)
    /// 
    ///     // build the container using the above parameters
    ///     .build()
    /// 
    ///         // do something with your container
    ///         .map(
    ///             |container| {
    ///             println!("container created!");
    ///             container
    ///         })
    /// 
    ///         // clean up your container when you're finished
    ///         .and_then(
    ///             |container| {
    ///             println!("removing container");
    /// 
    ///             container.delete()
    ///         })
    /// 
    ///         // handle any errors
    ///         .map_err(
    ///             |e| println!("Error: {}", e)
    ///         );
    /// 
    /// // run the future
    /// tokio::run(fut);
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
    pub fn delete(self) -> impl Future<Item = (), Error = shiplift::Error> {
        self.client
            .containers()
            .get(&self.id())
            .remove(RmContainerOptions::builder().force(true).build())
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
    pub fn build(self) -> impl Future<Item = Container, Error = shiplift::Error> {
        let image = self.image();
        let name = self.slugged_name();
        let ports = self.ports;

        if self.pull_on_build {
            future::Either::A(pull_image(self.client, image))
        } else {
            future::Either::B(future::ok((self.client, image)))
        }
        .and_then(|(client, image)| create_container(client, image, name, ports))
        .and_then(|(client, id)| run_container(client, id))
        .and_then(|(client, id)| inspect_container(client, id))
        .map(|(client, details)| Container { details, client })
    }
}

fn pull_image(
    client: Client,
    image: String,
) -> impl Future<Item = (Client, String), Error = shiplift::Error> {
    info!("pulling image: {}", &image);

    client
        .images()
        .pull(&PullOptions::builder().image(image.clone()).build())
        .for_each(|output| {
            debug!("{:?}", output);
            Ok(())
        })
        .map(move |_| {
            info!("pulled image: {}", &image);
            (client, image)
        })
}

fn create_container<S: AsRef<str>>(
    client: Client,
    image: String,
    container_name: Option<S>,
    ports: impl IntoIterator<Item = Port>,
) -> impl Future<Item = (Client, String), Error = shiplift::Error> {
    let mut container_options = ContainerOptions::builder(image.as_ref());

    if let Some(name) = container_name.as_ref() {
        container_options.name(name.as_ref());
    }

    for port in ports {
        container_options.expose(port.source, port.protocol.as_ref(), port.host);
    }

    client
        .containers()
        .create(&container_options.build())
        .map(|info| (client, info.id))
}

fn run_container(
    client: Client,
    id: String,
) -> impl Future<Item = (Client, String), Error = shiplift::Error> {
    client.containers().get(&id).start().map(|_| (client, id))
}

fn inspect_container(
    client: Client,
    id: String,
) -> impl Future<Item = (Client, ContainerDetails), Error = shiplift::Error> {
    client
        .containers()
        .get(&id)
        .inspect()
        .map(|details| (client, details))
}
