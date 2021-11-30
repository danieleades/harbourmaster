
# harbourmaster

[![Continuous integration](https://github.com/danieleades/harbourmaster/actions/workflows/CI.yml/badge.svg)](https://github.com/danieleades/harbourmaster/actions/workflows/CI.yml)
[![Latest Docs](https://docs.rs/harbourmaster/badge.svg)](https://docs.rs/harbourmaster/)

Harbourmaster is a library of high-level abstractions of Docker objects.

Harbourmaster is built on top of the excellent '[shiplift](https://github.com/softprops/shiplift)', but provides an object-oriented interface that
is a little easier to work with for some use cases.

Particularly useful for unit testing that involves spinning up and then
removing Docker containers.

### Usage
```rust
use harbourmaster::Container;

#[tokio::main]
async fn main() {
    let image = "alpine";

    println!("creating container!");
    let container = Container::new(image).await.unwrap();
    println!("container created!");

    println!("removing container!");
    container.delete().await.unwrap();
    println!("container removed!");
}
```

---

Current version: 0.5.0

License: Apache-2.0
