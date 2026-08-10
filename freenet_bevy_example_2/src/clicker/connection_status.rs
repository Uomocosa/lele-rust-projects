#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ConnectionStatus {
    #[default]
    Connecting,
    Connected,
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::ConnectionStatus;

    #[test]
    fn test_usage() {
        assert_eq!(ConnectionStatus::default(), ConnectionStatus::Connecting);
        assert_ne!(ConnectionStatus::Connected, ConnectionStatus::Connecting);
    }
}
