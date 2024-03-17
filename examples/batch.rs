use rqlite_rs::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = RqliteClientBuilder::new()
        .known_host("localhost:4001")
        .build()?;

    let q0 = rqlite_rs::query!("DROP TABLE IF EXISTS foo")?;
    let q1 =
        rqlite_rs::query!("CREATE TABLE IF NOT EXISTS foo (id INTEGER PRIMARY KEY, name TEXT)")?;
    let q2 = rqlite_rs::query!("INSERT INTO foo (name) VALUES (?)", "bar")?;
    let q3 = rqlite_rs::query!("SELECT * FROM foo")?;

    let batch = vec![q0, q1, q2, q3];

    let results = client.batch(batch).await?;

    for result in results {
        println!("{:?}", result);
    }

    Ok(())
}
