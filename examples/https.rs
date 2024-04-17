use rqlite_rs::{config::Scheme, prelude::*};

#[derive(FromRow)]
pub struct Table {
    name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = RqliteClientBuilder::new()
        .known_host("localhost:4001")
        .scheme(Scheme::Https)
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
