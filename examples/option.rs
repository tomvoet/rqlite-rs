use rqlite_rs::prelude::*;

#[derive(FromRow)]
pub struct OptionalRow {
    id: i32,
    optional: Option<i32>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = RqliteClientBuilder::new()
        .known_host("localhost:4001")
        .build()?;

    let drop_table_query = rqlite_rs::query!("DROP TABLE IF EXISTS test")?;

    client.exec(drop_table_query).await?;

    let create_table_query = rqlite_rs::query!(
        "CREATE TABLE IF NOT EXISTS test (id INTEGER PRIMARY KEY, optional INTEGER)"
    )?;

    client.exec(create_table_query).await?;

    let example_rows = [
        OptionalRow {
            id: 1,
            optional: None,
        },
        OptionalRow {
            id: 2,
            optional: Some(2),
        },
    ];

    let queries = example_rows
        .iter()
        .map(|row| {
            rqlite_rs::query!(
                "INSERT INTO test (id, optional) VALUES (?, ?)",
                row.id,
                row.optional
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    let res = client.batch(queries).await?;

    println!("Batch result: {:?}", res);

    let select_query = rqlite_rs::query!("SELECT * FROM test")?;

    let rows = client
        .fetch(select_query)
        .await?
        .into_typed::<OptionalRow>()?;

    rows.iter().for_each(|row| {
        println!("ID: {}, Optional: {:?}", row.id, row.optional);
    });

    let drop_table_query = rqlite_rs::query!("DROP TABLE test")?;

    client.exec(drop_table_query).await?;

    println!("Successfully cleaned up");

    Ok(())
}
