use super::FallbackStrategy;

/// A round-robin strategy for fallback.
/// This strategy will try the next host in the list.
/// The list is taken as passed to the `RqliteClientBuilder`.
pub struct RoundRobin;

impl RoundRobin {
    fn shift_hosts(hosts: &mut Vec<String>) {
        let host = hosts.remove(0);
        hosts.push(host);
    }
}

impl FallbackStrategy for RoundRobin {
    fn fallback<'a>(
        &mut self,
        hosts: &'a mut Vec<String>,
        current_host: &str,
        persist: bool,
    ) -> Option<&'a String> {
        if persist {
            Self::shift_hosts(hosts);
            Some(&hosts[0])
        } else {
            let index = hosts.iter().position(|host| host == current_host)?;
            let next_index = (index + 1) % hosts.len();
            Some(&hosts[next_index])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_fallback_round_robin() {
        let mut hosts = vec!["localhost:4001".to_string(), "localhost:4002".to_string()];
        let mut strategy = RoundRobin;

        assert_eq!(
            strategy.fallback(&mut hosts, "localhost:4001", false),
            Some(&"localhost:4002".to_string())
        );
        assert_eq!(
            hosts,
            vec!["localhost:4001".to_string(), "localhost:4002".to_string()]
        );
        assert_eq!(
            strategy.fallback(&mut hosts, "localhost:4002", false),
            Some(&"localhost:4001".to_string())
        );
        assert_eq!(
            hosts,
            vec!["localhost:4001".to_string(), "localhost:4002".to_string()]
        );
        assert_eq!(
            strategy.fallback(&mut hosts, "localhost:4001", true),
            Some(&"localhost:4002".to_string())
        );
        assert_eq!(
            hosts,
            vec!["localhost:4002".to_string(), "localhost:4001".to_string()]
        );
        assert_eq!(
            strategy.fallback(&mut hosts, "localhost:4001", false),
            Some(&"localhost:4002".to_string())
        );
        assert_eq!(
            hosts,
            vec!["localhost:4002".to_string(), "localhost:4001".to_string()]
        );
    }
}
