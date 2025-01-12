# rqlite-rs

[![build status](https://github.com/tomvoet/rqlite-rs/actions/workflows/ci.yml/badge.svg?branch=main&event=push)](https://github.com/tomvoet/rqlite-rs/actions/workflows/ci.yml) 
[![Test Coverage](https://codecov.io/gh/tomvoet/rqlite-rs/graph/badge.svg?token=T9TVDKKV3J)](https://codecov.io/gh/tomvoet/rqlite-rs)
[![Crates.io](https://img.shields.io/crates/v/rqlite-rs.svg)](https://crates.io/crates/rqlite-rs)
[![Documentation](https://docs.rs/rqlite-rs/badge.svg)](https://docs.rs/rqlite-rs)
[![License](https://img.shields.io/crates/l/rqlite-rs.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/d/rqlite-rs.svg)](https://crates.io/crates/rqlite-rs)
![Rust](https://img.shields.io/badge/rust-1.71.1%2B-blue.svg)

**rqlite-rs** is a Rust client for [rqlite](https://rqlite.io/), the distributed relational database built on SQLite, providing an async interface for seamless integration with Rust's async ecosystems. Utilizing [reqwest](https://crates.io/crates/reqwest) for efficient connection management, it offers a Rustic, high-level API for easy and efficient interaction with rqlite clusters.

## Features

- **Asynchronous Interface**: Fully async, compatible with Tokio, async-std, and smol.
- **Connection Pooling**: Efficient management of connections to the rqlite cluster.
- **High-Level API**: Simplifies interactions with the rqlite API.
- **Resilience**: Automatic failover to a secondary node on connectivity issues.
- **Cluster Management**: Full control over the cluster with node query and management features.

## Installation

Add to your `Cargo.toml`:

```diff
[dependencies]
...
+ rqlite-rs = "0.6.0"
```

## Quick Start

Ensure you have a running rqlite cluster. Replace `localhost:4001` and `localhost:4002` with your node addresses:

```rust
use rqlite_rs::prelude::*;

#[derive(FromRow)]
pub struct Table {
    name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = RqliteClientBuilder::new()
        .known_host("localhost:4001")
        .build()?;

    let query = rqlite_rs::query!(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
    )?;

    let rows = client.fetch(query).await?;

    let tables = rows.into_typed::<Table>()?;

    for table in tables {
        println!("Table: {}", table.name);
    }

    Ok(())
}
```

## Resilience

**rqlite-rs** supports automatic failover to a different node in the cluster. This can be done using one of the provided fallback strategies (e.g., `Random`, `RoundRobin`, `Priority`).
Furthermore you can also implement your own fallback strategy by implementing the `FallbackStrategy` trait. An example of this can be found in the [custom_fallback](https://github.com/tomvoet/rqlite-rs/blob/main/examples/custom-fallback.rs) example.

## Documentation

For detailed API documentation and advanced usage, visit [rqlite-rs documentation](https://docs.rs/rqlite-rs/).

## Contributing

Contributions are welcome!

## License

**rqlite-rs** is licensed under either of
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
- MIT license ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.