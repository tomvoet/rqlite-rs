use rqlite_rs::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = RqliteClientBuilder::new("localhost:4001").build()?;

    let nodes = client.nodes().await?;

    for node in nodes {
        println!("{:?}", node);
    }

    Ok(())
}
