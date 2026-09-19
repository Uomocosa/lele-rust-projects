use freenet_libp2p_bevy_plugin::{net_id, p2p};

use crate::clicker;

#[must_use]
pub fn move_gossip(from: &str, owner: u64) -> p2p::Event<clicker::CursorMsg> {
    let msg = clicker::CursorMsg::Move {
        owner: net_id::NetworkId(owner),
        pos: [30.0, 40.0],
    };
    let data = bincode::serialize(&msg).unwrap_or_default();
    p2p::Event::Gossip {
        topic: clicker::pos_topic(&clicker::ActiveLobby("alpha".to_string())),
        from: from.to_string(),
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::move_gossip;
    use freenet_libp2p_bevy_plugin::p2p;

    #[test]
    fn test_usage() {
        assert!(matches!(
            move_gossip("peer-2", 2),
            p2p::Event::Gossip { .. }
        ));
    }
}
