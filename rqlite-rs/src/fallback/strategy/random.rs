use nanorand::Rng;

use super::FallbackStrategy;

/// Random is a `FallbackStrategy` that selects a random host from the list of hosts.
/// This strategy will try a random host from the list.
/// The list is taken as passed to the `RqliteClientBuilder`.
pub struct Random {
    rng: nanorand::WyRand,
}

impl Default for Random {
    fn default() -> Self {
        Self::new()
    }
}

impl Random {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rng: nanorand::WyRand::new(),
        }
    }

    #[must_use]
    pub fn new_seed(seed: u64) -> Self {
        Self {
            rng: nanorand::WyRand::new_seed(seed),
        }
    }
}

impl FallbackStrategy for Random {
    fn fallback<'a>(
        &mut self,
        hosts: &'a mut Vec<String>,
        _current_host: &str,
        persist: bool,
    ) -> Option<&'a String> {
        let index = self.rng.generate_range(0..hosts.len());
        println!("persist: {persist}");
        if persist {
            hosts.swap(0, index);
            Some(&hosts[0])
        } else {
            Some(&hosts[index])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_fallback_random() {
        let mut hosts = vec!["localhost:4001".to_string(), "localhost:4002".to_string()];
        let mut strategy = Random::new();

        assert!(strategy
            .fallback(&mut hosts, "localhost:4001", false)
            .is_some());
        assert!(strategy
            .fallback(&mut hosts, "localhost:4002", false)
            .is_some());
        assert!(strategy
            .fallback(&mut hosts, "localhost:4001", true)
            .is_some());
    }

    #[test]
    fn unit_fallback_random_default() {
        let mut hosts = vec!["localhost:4001".to_string(), "localhost:4002".to_string()];
        let mut strategy = Random::default();

        assert!(strategy
            .fallback(&mut hosts, "localhost:4001", false)
            .is_some());
        assert!(strategy
            .fallback(&mut hosts, "localhost:4002", false)
            .is_some());
        assert!(strategy
            .fallback(&mut hosts, "localhost:4001", true)
            .is_some());
    }

    #[test]
    fn unit_fallback_random_seed() {
        let mut hosts = vec!["localhost:4001".to_string(), "localhost:4002".to_string()];
        let mut strategy = Random::new_seed(42);

        assert!(strategy
            .fallback(&mut hosts, "localhost:4001", false)
            .is_some());
        assert!(strategy
            .fallback(&mut hosts, "localhost:4002", false)
            .is_some());
        assert!(strategy
            .fallback(&mut hosts, "localhost:4001", true)
            .is_some());
    }
}
