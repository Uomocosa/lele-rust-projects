#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiscoveryTiming {
    pub tick_secs: u64,
    pub timeout_secs: u64,
}

impl Default for DiscoveryTiming {
    fn default() -> Self {
        Self {
            tick_secs: 5,
            timeout_secs: 300,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DiscoveryTiming;

    #[test]
    fn test_usage() {
        let timing = DiscoveryTiming::default();
        assert_eq!(timing.tick_secs, 5);
        assert_eq!(timing.timeout_secs, 300);
    }
}
