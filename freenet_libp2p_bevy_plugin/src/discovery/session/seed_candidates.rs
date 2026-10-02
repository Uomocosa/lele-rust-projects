use crate::discovery;
use discovery::session::Session;

pub fn seed_candidates(session: &mut Session) {
    let Some(room) = &session.room else {
        return;
    };
    let Some(record) = session.directory.get(&room.name) else {
        return;
    };
    let seeds: Vec<_> = record
        .members
        .iter()
        .filter(|(peer, presence)| **peer != session.me && !presence.addrs.is_empty())
        .map(|(peer, presence)| (peer.clone(), presence.clone()))
        .collect();
    for (peer, presence) in seeds {
        let newer = session
            .candidates
            .get(&peer)
            .is_none_or(|known| known.updated_at < presence.updated_at);
        if newer {
            session.candidates.insert(peer, presence);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::seed_candidates;
    use crate::discovery;
    use discovery::session::Session;

    #[test]
    fn test_usage() {
        let mut session = Session::new(
            discovery::PeerId("me".to_string()),
            Vec::new(),
            discovery::Timing::default(),
        );
        let room = discovery::RoomName("r".to_string());
        session.room = Some(discovery::Room {
            name: room.clone(),
            members: discovery::Members::new(),
        });
        let presence = discovery::Presence {
            addrs: vec!["/ip4/1".to_string()],
            updated_at: discovery::EpochSecs(5),
        };
        let members = BTreeMap::from([
            (discovery::PeerId("me".to_string()), presence.clone()),
            (discovery::PeerId("a".to_string()), presence),
        ]);
        session.directory.insert(
            room,
            discovery::RoomRecord {
                capacity: 8,
                members,
            },
        );
        seed_candidates(&mut session);
        let seeded: Vec<_> = session.candidates.keys().cloned().collect();
        assert_eq!(seeded, vec![discovery::PeerId("a".to_string())]);
    }
}
