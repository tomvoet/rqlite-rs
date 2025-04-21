mod round_robin;
pub use round_robin::RoundRobin;
mod priority;
pub use priority::Priority;
#[cfg(feature = "random-fallback")]
mod random;
#[cfg(feature = "random-fallback")]
pub use random::Random;

/// `FallbackStrategy` is the trait that defines the strategy to use when a host fails.
/// The default strategy is `RoundRobin`.
pub trait FallbackStrategy: Send + Sync + 'static {
    /// fallback returns the next host to try and can modify the hosts list if needed.
    fn fallback<'a>(
        &mut self,
        hosts: &'a mut Vec<String>,
        current_host: &str,
        persist: bool,
    ) -> Option<&'a String>;
}

impl Default for Box<dyn FallbackStrategy> {
    fn default() -> Box<dyn FallbackStrategy> {
        Box::new(round_robin::RoundRobin)
    }
}
