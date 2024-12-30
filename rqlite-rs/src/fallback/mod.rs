mod strategy;
pub use strategy::*;

/// `FallbackCount` is the number of hosts to try to fallback to if the current host fails.
#[derive(Default)]
pub enum FallbackCount {
    /// Equivalent to the current number of hosts.
    #[default]
    NumHosts,
    /// None means no fallback.
    None,
    /// A specific number of hosts to fallback to.
    /// If the number is greater than the number of hosts, it will fallback to all hosts.
    Count(usize),
    /// A percentage of the total number of hosts to fallback to.
    /// If the percentage is greater than 100, it will fallback to all hosts.
    Percentage(u8),
    /// Never stop trying to fallback.
    /// This is useful for testing purposes.
    Infinite,
}

impl FallbackCount {
    pub(crate) fn count(&self, hosts: usize) -> usize {
        match self {
            FallbackCount::NumHosts => hosts,
            FallbackCount::None => 0,
            FallbackCount::Count(count) => *count,
            FallbackCount::Percentage(percentage) => {
                #[allow(
                    clippy::cast_sign_loss,
                    clippy::cast_possible_truncation,
                    clippy::cast_precision_loss
                )]
                let count = (hosts as f64 * (f64::from(*percentage) / 100.0)).ceil() as usize;
                if count > hosts {
                    hosts
                } else {
                    count
                }
            }
            FallbackCount::Infinite => usize::MAX,
        }
    }
}
