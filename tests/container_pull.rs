use harbourmaster::Container;

#[tokio::test]
async fn main() {
    let container = Container::pull("alpine").await.unwrap();
    // clean up container
    container.delete().await.unwrap();
}
