# rqlite-rs

[![Crates.io](https://img.shields.io/crates/v/rqlite-rs.svg)](https://crates.io/crates/rqlite-rs)
[![Documentation](https://docs.rs/rqlite-rs/badge.svg)](https://docs.rs/rqlite-rs)
[![License](https://img.shields.io/crates/l/rqlite-rs.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/d/rqlite-rs.svg)](https://crates.io/crates/rqlite-rs)
![Rust](https://img.shields.io/badge/rust-1.40%2B-blue.svg)

**rqlite-rs** is a Rust client for [rqlite](https://rqlite.io/), the distributed relational database built on SQLite, providing an async interface for seamless integration with Rust's async ecosystems. Utilizing [reqwest](https://crates.io/crates/reqwest) for efficient connection management, it offers a Rustic, high-level API for easy and efficient interaction with rqlite clusters.

## Features

- **Asynchronous Interface**: Fully async, compatible with Tokio, async-std, and smol.
- **Connection Pooling**: Efficient management of connections to the rqlite cluster.
- **High-Level API**: Simplifies interactions with the rqlite API.
- **Resilience**: Automatic failover to a secondary node on connectivity issues.
- **Cluster Management**: Full control over the cluster with node query and management features.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rqlite-rs = "0.1.0"
```

## Quick Start

Ensure you have a running rqlite cluster. Replace `localhost:4001` and `localhost:4002` with your node addresses:

```rust
use rqlite_rs::prelude::*;

#[derive(FromRow)]
pub struct Table { name: String, }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = RqliteClientBuilder::new()
        .known_host("localhost:4001")
        .known_host("localhost:4002")
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

The client supports automatic failover, attempting to connect to the next known node if a connection error or timeout occurs, ensuring high availability.

## Documentation

For detailed API documentation and advanced usage, visit [rqlite-rs documentation](https://docs.rs/rqlite-rs/).

## Contributing

Contributions are welcome!

## License

**rqlite-rs** is licensed under the MIT license. See [LICENSE](LICENSE) for details.
