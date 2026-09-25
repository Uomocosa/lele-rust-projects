#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiscoveryStatus {
    #[default]
    Known,
    Connecting,
    Connected,
}

#[cfg(test)]
mod tests {
    use super::DiscoveryStatus;

    #[test]
    fn test_usage() {
        assert_eq!(DiscoveryStatus::default(), DiscoveryStatus::Known);
        assert_ne!(DiscoveryStatus::Known, DiscoveryStatus::Connected);
    }
}
