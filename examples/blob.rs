use rqlite_rs::prelude::*;

#[derive(FromRow, Debug)]
pub struct Table {
    id: i64,
    blob: Vec<u8>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = RqliteClientBuilder::new()
        .known_host("localhost:4001")
        .build()?;

    let drop_table = rqlite_rs::query!("DROP TABLE IF EXISTS blob_table")?;

    client.exec(drop_table).await?;

    let create_table = rqlite_rs::query!(
        "CREATE TABLE IF NOT EXISTS blob_table (id INTEGER PRIMARY KEY, blob BLOB)"
    )?;

    client.exec(create_table).await?;

    let insert_blob = rqlite_rs::query!(
        "INSERT INTO blob_table (id, blob) VALUES (?, ?)",
        1,
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    )?;

    client.exec(insert_blob).await?;

    let select_blob = rqlite_rs::query!("SELECT * FROM blob_table WHERE id = ?", 1)?;

    let rows = client.fetch(select_blob).await?.into_typed::<Table>()?;

    let row = rows.first().unwrap();

    println!("{:?}", row);

    assert_eq!(row.id, 1);
    assert_eq!(row.blob, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

    let drop_table_query = rqlite_rs::query!("DROP TABLE blob_table")?;

    client.exec(drop_table_query).await?;

    println!("Successfully cleaned up");

    Ok(())
}
