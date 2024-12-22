An async rqlite client for Rust.

Have a look at the [README](https://www.github.com/tomvoet/rqlite-rs/tree/main/README.md) to get started or [browse the examples](https://www.github.com/tomvoet/rqlite-rs/tree/main/examples).

The simplest way to get started is importing the [`prelude`] and creating a client via the [`RqliteClientBuilder`]:

```ignore
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

This will print all tables in the rqlite database.

## Features

- **macros**: Use the `FromRow` derive macro to automatically convert rows to structs.
- **fast-blob**: When enabled, the client will use base64 encoding for retrieving blobs, reducing the amount of data transferred.