use crate::discovery;
use crate::net_id;
use discovery::session::Session;

pub fn add_candidates(
    session: &mut Session,
    peers: impl IntoIterator<Item = (net_id::PeerId, Vec<net_id::PeerAddr>)>,
    now: discovery::EpochSecs,
) {
    for (peer, addrs) in peers {
        if peer == session.me || peer.is_empty() || addrs.is_empty() {
            continue;
        }
        session.candidates.insert(
            peer,
            discovery::Presence {
                addrs,
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
            net_id::PeerId::from("me"),
            Vec::new(),
            discovery::Timing::default(),
        );
        let peers = vec![
            (
                net_id::PeerId::from("me"),
                vec![net_id::PeerAddr::from("/ip4/1")],
            ),
            (
                net_id::PeerId::from("a"),
                vec![net_id::PeerAddr::from("/ip4/2")],
            ),
            (net_id::PeerId::from("b"), Vec::new()),
        ];
        add_candidates(&mut session, peers, discovery::EpochSecs(1));
        assert_eq!(session.candidates.len(), 1);
        assert!(session.candidates.contains_key(&net_id::PeerId::from("a")));
    }
}
