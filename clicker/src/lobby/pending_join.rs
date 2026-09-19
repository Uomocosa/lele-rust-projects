use freenet_libp2p_bevy_plugin::net_id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingJoin {
    pub peer: String,
    pub joiner: net_id::NetworkId,
    pub reveal_at: std::time::Instant,
}

#[cfg(test)]
mod tests {
    use super::PendingJoin;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let entry = PendingJoin {
            peer: "peer-2".to_string(),
            joiner: net_id::NetworkId(2),
            reveal_at: std::time::Instant::now(),
        };
        assert_eq!(entry.peer, "peer-2");
        assert_eq!(*entry.joiner, 2);
    }
}
