use super::FallbackStrategy;

/// A priority strategy for fallback.
/// This strategy will try the next host in the list.
/// The list can be passed to the `Priority::new` function.
/// If no list is passed, the hosts will be used in the order they were passed.
/// 
/// # Example
/// ```
/// use rqlite_rs::{fallback::{FallbackCount, Priority}, RqliteClientBuilder};
/// 
/// let client = RqliteClientBuilder::new()
///     .known_host("localhost:4005")
///     .known_host("localhost:4003")
///     .known_host("localhost:4001")
///     .fallback_strategy(Priority::new(vec![
///         "localhost:4005".to_string(),
///         "localhost:4003".to_string(),
///         "localhost:4001".to_string(),
///     ]))
///     .build();
/// 
/// assert!(client.is_ok());
/// ```
pub struct Priority {
    hosts: Vec<String>,
}

impl Priority {
    pub fn new(hosts: Vec<String>) -> Priority {
        Priority {
            hosts,
        }
    }
}

impl FallbackStrategy for Priority {
    fn fallback<'a>(&mut self, hosts: &'a mut Vec<String>, current_host: &str, persist: bool) -> Option<&'a String> {
        // If hosts is empty, assume that the hosts were passed in order of priority.
        if self.hosts.is_empty() {
            self.hosts = hosts.clone();
        }

        let Some(index) = self.hosts.iter().position(|host| host == current_host) else {
            return None;
        };

        let next_index = (index + 1) % self.hosts.len();
        
        if persist {
            hosts.swap(0, next_index);
            Some(&hosts[0])
        } else {
            Some(&hosts[next_index])
        }
    }
}