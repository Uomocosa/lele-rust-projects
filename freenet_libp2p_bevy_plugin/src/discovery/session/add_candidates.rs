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
            net_id::PeerId("me".to_string()),
            Vec::new(),
            discovery::Timing::default(),
        );
        let peers = vec![
            (
                net_id::PeerId("me".to_string()),
                vec![net_id::PeerAddr("/ip4/1".to_string())],
            ),
            (
                net_id::PeerId("a".to_string()),
                vec![net_id::PeerAddr("/ip4/2".to_string())],
            ),
            (net_id::PeerId("b".to_string()), Vec::new()),
        ];
        add_candidates(&mut session, peers, discovery::EpochSecs(1));
        assert_eq!(session.candidates.len(), 1);
        assert!(
            session
                .candidates
                .contains_key(&net_id::PeerId("a".to_string()))
        );
    }
}
