#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportMode {
    Tcp,
    Quic,
    Both,
}

#[cfg(test)]
mod tests {
    use super::TransportMode;

    #[test]
    fn test_usage() {
        assert_ne!(TransportMode::Tcp, TransportMode::Quic);
        assert_eq!(TransportMode::Both, TransportMode::Both);
    }
}
