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
    /// If the number is greater than the total number of hosts, it can lead to hosts being tried multiple times.
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
            Self::NumHosts => hosts,
            Self::None => 0,
            Self::Count(count) => *count,
            Self::Percentage(percentage) => {
                #[expect(
                    clippy::cast_sign_loss,
                    clippy::cast_possible_truncation,
                    clippy::cast_precision_loss,
                    reason = "percentage calculation with controlled input values"
                )]
                let count = (hosts as f64 * (f64::from(*percentage) / 100.0)).ceil() as usize;
                if count > hosts {
                    hosts
                } else {
                    count
                }
            }
            Self::Infinite => usize::MAX,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_fallback_count() {
        assert_eq!(FallbackCount::NumHosts.count(3), 3);
        assert_eq!(FallbackCount::None.count(3), 0);
        assert_eq!(FallbackCount::Count(2).count(3), 2);
        assert_eq!(FallbackCount::Count(4).count(3), 4);
        assert_eq!(FallbackCount::Percentage(50).count(3), 2);
        assert_eq!(FallbackCount::Percentage(200).count(3), 3);
        assert_eq!(FallbackCount::Infinite.count(3), usize::MAX);
    }
}
