use freenet_libp2p_bevy_plugin::p2p;

use crate::clicker;
use crate::discovery;

#[must_use]
pub fn gossip_roster(
    from: &str,
    owner: u64,
    peer_id: &str,
    addrs: Vec<String>,
    updated_at: u64,
) -> p2p::Event<clicker::CursorMsg> {
    let mut state = discovery::RosterState::new();
    state.insert(
        discovery::PlayerId(owner),
        discovery::PeerEntry {
            peer_id: peer_id.to_string(),
            addrs,
            updated_at,
        },
    );
    p2p::Event::Gossip {
        topic: "clicker/alpha/roster".to_string(),
        from: from.to_string(),
        data: bincode::serialize(&state).unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::gossip_roster;
    use freenet_libp2p_bevy_plugin::p2p;

    #[test]
    fn test_usage() {
        assert!(matches!(
            gossip_roster("peer-2", 3, "peer-3", Vec::new(), 1),
            p2p::Event::Gossip { .. }
        ));
    }
}
