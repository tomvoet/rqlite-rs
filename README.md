# rqlite-rs

This is a rust client for [rqlite](https://rqlite.io/).
It is written fully in rust, provides a fully async interface using [reqwest](https://crates.io/crates/reqwest), which allows for easy integration with async runtimes like tokio, async-std, smol, etc.
Utilizing a reqwest connection pool it can efficiently manage connections to the rqlite cluster.
It is fully compatible with the rqlite API and provides a high level interface for easy usage.

## Usage
```rust
use rqlite_rs::prelude::*;

#[derive(FromRow)]
pub struct Table {
    name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = RqLiteClientBuilder::new("localhost:4001").build()?;

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


## Roadmap
- [] Automtically switch to different node if the current node is down