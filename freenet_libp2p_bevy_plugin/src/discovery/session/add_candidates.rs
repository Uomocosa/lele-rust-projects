use crate::discovery;
use crate::net_id;
use discovery::session::Session;

pub fn add_candidates(
    session: &mut Session,
    peers: impl IntoIterator<Item = net_id::Peer>,
    now: discovery::EpochSecs,
) {
    for peer in peers {
        if peer.id == session.me.id || peer.id.is_empty() || peer.addrs.is_empty() {
            continue;
        }
        session.candidates.insert(
            peer.id,
            discovery::Presence {
                addrs: peer.addrs,
                updated_at: now,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::add_candidates;
    use crate::discovery;
    use crate::net_id;
    use discovery::session::Session;

    #[test]
    fn test_usage() {
        let mut session = Session::new(
            net_id::Peer {
                id: net_id::PeerId::from("me"),
                addrs: Vec::new(),
            },
            discovery::Timing::default(),
        );
        let peers = vec![
            net_id::Peer {
                id: net_id::PeerId::from("me"),
                addrs: vec![net_id::PeerAddr::from("/ip4/1")],
            },
            net_id::Peer {
                id: net_id::PeerId::from("a"),
                addrs: vec![net_id::PeerAddr::from("/ip4/2")],
            },
            net_id::Peer {
                id: net_id::PeerId::from("b"),
                addrs: Vec::new(),
            },
        ];
        add_candidates(&mut session, peers, discovery::EpochSecs(1));
        assert_eq!(session.candidates.len(), 1);
        assert!(session.candidates.contains_key(&net_id::PeerId::from("a")));
    }
}
