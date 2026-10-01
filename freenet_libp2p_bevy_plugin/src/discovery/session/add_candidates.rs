use crate::discovery;
use discovery::id::{EpochSecs, Presence, RemotePeerId};
use discovery::session::Session;

pub fn add_candidates(
    session: &mut Session,
    peers: impl IntoIterator<Item = (RemotePeerId, Vec<String>)>,
    now: EpochSecs,
) {
    for (peer, addrs) in peers {
        if peer == session.me || peer.is_empty() || addrs.is_empty() {
            continue;
        }
        session.candidates.insert(
            peer,
            Presence {
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
    use discovery::id::{EpochSecs, RemotePeerId};

    #[test]
    fn test_usage() {
        let mut session = discovery::session::Session::new(
            discovery::id::UniqueGameId::new(
                &discovery::id::GameName("g".to_string()),
                &discovery::id::GameToken("t".to_string()),
            ),
            RemotePeerId("me".to_string()),
            Vec::new(),
            8,
        );
        let peers = vec![
            (RemotePeerId("me".to_string()), vec!["/ip4/1".to_string()]),
            (RemotePeerId("a".to_string()), vec!["/ip4/2".to_string()]),
            (RemotePeerId("b".to_string()), Vec::new()),
        ];
        add_candidates(&mut session, peers, EpochSecs(1));
        assert_eq!(session.candidates.len(), 1);
        assert!(
            session
                .candidates
                .contains_key(&RemotePeerId("a".to_string()))
        );
    }
}
