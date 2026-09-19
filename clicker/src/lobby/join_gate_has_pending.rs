use crate::lobby;
use freenet_libp2p_bevy_plugin::net_id;

pub fn has_pending(gate: &lobby::JoinGate, joiner: net_id::NetworkId) -> bool {
    gate.pending.iter().any(|held| held.joiner == joiner)
}

#[cfg(test)]
mod tests {
    use super::has_pending;
    use crate::lobby;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let mut gate = lobby::JoinGate::default();
        assert!(!has_pending(&gate, net_id::NetworkId(2)));
        gate.pending.push(lobby::PendingJoin {
            peer: "peer-2".to_string(),
            joiner: net_id::NetworkId(2),
            reveal_at: std::time::Instant::now(),
        });
        assert!(has_pending(&gate, net_id::NetworkId(2)));
    }
}
