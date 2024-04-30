use rqlite_rs::prelude::*;

#[derive(FromRow)]
pub struct Table {
    name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = RqliteClientBuilder::new()
        .known_host("localhost:4001")
        .auth("tango", "test")
        .build()?;

    let query = rqlite_rs::query!(
        "insert into container_images values('y2Set5DHfrhDW6H6lfs8D2PFAR-BXVLs', 'alpine', 'latest', current_timestamp)"
    )?;

    let rows = client.exec(query).await?;
    
    Ok(())
}
