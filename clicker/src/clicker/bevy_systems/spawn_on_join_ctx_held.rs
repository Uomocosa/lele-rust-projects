use crate::lobby;
use freenet_libp2p_bevy_plugin::net_id;

#[must_use]
pub fn held(
    own: net_id::NetworkId,
    pending: &[lobby::PendingJoin],
    absent: &[String],
    peer: &str,
    id: net_id::NetworkId,
) -> bool {
    id == own
        || absent.iter().any(|entry| entry == peer)
        || pending
            .iter()
            .any(|entry| entry.peer == peer || entry.joiner == id)
}

#[cfg(test)]
mod tests {
    use super::held;
    use crate::lobby;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let own = net_id::NetworkId(1);
        assert!(held(own, &[], &[], "peer-1", own), "own is always held");
        assert!(!held(own, &[], &[], "peer-2", net_id::NetworkId(2)));
        let pending = vec![lobby::PendingJoin {
            peer: "peer-2".to_string(),
            joiner: net_id::NetworkId(2),
            reveal_at: std::time::Instant::now(),
            fail_at: std::time::Instant::now(),
        }];
        assert!(held(own, &pending, &[], "peer-2", net_id::NetworkId(2)));
        assert!(held(own, &pending, &[], "other-peer", net_id::NetworkId(2)));
        let absent = vec!["peer-3".to_string()];
        assert!(held(own, &[], &absent, "peer-3", net_id::NetworkId(3)));
    }
}
