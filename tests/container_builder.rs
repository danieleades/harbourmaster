use harbourmaster::{Container, Protocol};

#[tokio::test]
async fn main() {
    let container = Container::builder("couchdb")
        // the docker image tag to use
        .tag("2.3.0")

        // set the name of the docker container
        .name("test_container")

        // add environment variables
        .environment_variable("COUCHDB_USER=admin")
        .environment_variable("COUCHDB_PASSWORD=password")

        // optionally add a randomised alphanumeric 'slug' to the
        // container name. Useful if you're creating and
        // naming them in bulk
        .slug_length(6)

        // expose ports on the container to the host machine
        .expose(5984, 5984, Protocol::Tcp)

        // if set, pull the image from the webular information
        // super-highway before building.
        .pull_on_build()

        // build the container using the above parameters
        .build()
        .await
        .unwrap();

    println!("container created!");
    container.delete().await.unwrap();
    println!("container deleted!");
}
