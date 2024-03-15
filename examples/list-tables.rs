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
