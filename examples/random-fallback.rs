use rqlite_rs::{fallback::{FallbackCount, Random}, RqliteClientBuilder};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = RqliteClientBuilder::new()
        .known_host("localhost:4011")
        .known_host("localhost:4009")
        .known_host("localhost:4007")
        .known_host("localhost:4005")
        .known_host("localhost:4003")
        .known_host("localhost:4001")
        .fallback_strategy(Random::new_seed(42))
        .fallback_count(FallbackCount::Infinite)
        .fallback_persistence(false)
        .build()?;

    loop {
        let query = rqlite_rs::query!(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
        )?;

        let rows = client.fetch(query).await;

        let status = match rows {
            Ok(_) => "OK".to_string(),
            Err(e) => e.to_string(),
        };

        println!("Status: {}", status);

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}