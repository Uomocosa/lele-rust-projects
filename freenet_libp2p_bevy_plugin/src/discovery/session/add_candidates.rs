use crate::discovery;
use discovery::session::Session;

pub fn add_candidates(
    session: &mut Session,
    peers: impl IntoIterator<Item = (discovery::PeerId, Vec<String>)>,
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
    use discovery::session::Session;

    #[test]
    fn test_usage() {
        let mut session = Session::new(
            discovery::PeerId("me".to_string()),
            Vec::new(),
            discovery::Timing::default(),
        );
        let peers = vec![
            (
                discovery::PeerId("me".to_string()),
                vec!["/ip4/1".to_string()],
            ),
            (
                discovery::PeerId("a".to_string()),
                vec!["/ip4/2".to_string()],
            ),
            (discovery::PeerId("b".to_string()), Vec::new()),
        ];
        add_candidates(&mut session, peers, discovery::EpochSecs(1));
        assert_eq!(session.candidates.len(), 1);
        assert!(
            session
                .candidates
                .contains_key(&discovery::PeerId("a".to_string()))
        );
    }
}
