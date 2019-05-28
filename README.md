
# harbourmaster

[![Build Status](https://travis-ci.org/danieleades/harbourmaster.svg?branch=master)](https://travis-ci.org/danieleades/harbourmaster)
[![Latest Docs](https://docs.rs/harbourmaster/badge.svg)](https://docs.rs/harbourmaster/)

Harbourmaster is a library of high-level abstractions of Docker objects.

Harbourmaster is built on top of the excellent '[shiplift](https://github.com/softprops/shiplift)', but provides an object-oriented interface that
is a little easier to work with for some use cases.

Particularly useful for unit testing that involves spinning up and then removing Docker containers.

## Usage
```rust
use tokio::prelude::Future;
use harbourmaster::Container;

let image = "alpine";

let fut = Container::new(image)
    .map(
        |container| {
        println!("container created!");
        container
    })
    .and_then(
        |container| {
        println!("removing container");
        container.delete()
    })
    .map_err(
        |e| println!("Error: {}", e)
    );

tokio::run(fut);
```


---

Current version: 0.1.0

License: Apache-2.0
