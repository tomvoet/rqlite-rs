#![cfg(feature = "random-fallback")]
use nanorand::Rng;

use super::FallbackStrategy;

/// Random is a FallbackStrategy that selects a random host from the list of hosts.
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
    pub fn new() -> Self {
        Self {
            rng: nanorand::WyRand::new(),
        }
    }

    pub fn new_seed(seed: u64) -> Self {
        Self {
            rng: nanorand::WyRand::new_seed(seed),
        }
    }
}

impl FallbackStrategy for Random {
    fn fallback<'a>(&mut self, hosts: &'a mut Vec<String>, _current_host: &str, persist: bool) -> Option<&'a String> {
        let index = self.rng.generate_range(0..hosts.len());
        println!("persist: {}", persist);
        if persist {
            hosts.swap(0, index);
            Some(&hosts[0])
        } else {
            Some(&hosts[index])
        }
    }
}