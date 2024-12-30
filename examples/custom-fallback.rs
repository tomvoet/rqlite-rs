use std::{collections::HashMap, sync::{Arc, Mutex}};

use rqlite_rs::{fallback::{FallbackCount, FallbackStrategy}, RqliteClientBuilder};

#[derive(Default)]
pub struct LeastFailuresFallback {
    failures: Arc<Mutex<HashMap<String, u32>>>,
}

impl FallbackStrategy for LeastFailuresFallback {
    fn fallback<'a>(&mut self, hosts: &'a mut Vec<String>, current_host: &str, persist: bool) -> Option<&'a String> {
        let mut failures = self.failures.lock().unwrap();

        // Initialize the failures map if it is empty
        if failures.is_empty() {
            for host in hosts.iter() {
                failures.insert(host.clone(), 0);
            }
        }

        // Print the failures map if needed
        // println!("failures: {:?}", failures);

        // Report the current host as failed
        if let Some(count) = failures.get_mut(current_host) {
            *count += 1;
        } else {
            failures.insert(current_host.to_string(), 1);
        }

        let mut sorted_hosts = hosts.clone();
        sorted_hosts.sort_by_key(|host| failures.get(host).cloned().unwrap_or(0));

        let next_host = sorted_hosts.iter().find(|&host| host != current_host)?;

        if persist {
            let index = hosts.iter().position(|host| host == next_host)?;
            hosts.swap(0, index);
            Some(&hosts[0])
        } else {
            hosts.iter().find(|&host| host == next_host)
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = RqliteClientBuilder::new()
        .known_host("localhost:4011")
        .known_host("localhost:4009")
        .known_host("localhost:4007")
        .known_host("localhost:4005")
        .known_host("localhost:4003")
        .known_host("localhost:4001")
        .fallback_strategy(LeastFailuresFallback::default())
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