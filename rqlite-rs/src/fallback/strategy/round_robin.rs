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
    fn fallback<'a>(&mut self, hosts: &'a mut Vec<String>, _current_host: &str, persist: bool) -> Option<&'a String> {
        if persist {
            Self::shift_hosts(hosts);
            Some(&hosts[0])
        } else {
            let index = hosts.iter().position(|host| host == _current_host)?;
            let next_index = (index + 1) % hosts.len();
            Some(&hosts[next_index])
        }
    }
}