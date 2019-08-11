
# harbourmaster

[![Build Status](https://travis-ci.org/danieleades/harbourmaster.svg?branch=master)](https://travis-ci.org/danieleades/harbourmaster)
[![Build Status](https://travis-ci.org/danieleades/harbourmaster.svg?branch=master)](https://travis-ci.org/danieleades/harbourmaster)
[![Latest Docs](https://docs.rs/harbourmaster/badge.svg)](https://docs.rs/harbourmaster/)

Harbourmaster is a library of high-level abstractions of Docker objects.

Harbourmaster is built on top of the excellent '[shiplift](https://github.com/softprops/shiplift)', but provides an object-oriented interface that
is a little easier to work with for some use cases. It's also using async/await-ready futures-0.3 for the interface.

Particularly useful for unit testing that involves spinning up and then removing Docker containers.

### Usage
```rust
use harbourmaster::Container;

let fut = async {
    let image = "alpine";

    println!("creating container!");
    let container = Container::new(image).await.unwrap();
    println!("container created!");

    println!("removing container!");
    container.delete().await.unwrap();
    println!("container removed!");
    };

// Currently, we must convert 0.3 future to 0.1 future to run on tokio executor
use futures::future::{FutureExt, TryFutureExt};
let future01 = fut.unit_error().boxed().compat();

tokio::run(future01);
```

---

Current version: 0.3.0

License: Apache-2.0
