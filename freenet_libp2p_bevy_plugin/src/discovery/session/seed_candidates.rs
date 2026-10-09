use crate::discovery;
use discovery::session::Session;

pub fn seed_candidates(session: &mut Session) {
    let Some(room) = &session.room else {
        return;
    };
    let Some(record) = session.lobby.get(&room.name) else {
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
    use crate::net_id;
    use std::collections::BTreeMap;

    use super::seed_candidates;
    use crate::discovery;
    use discovery::session::Session;

    #[test]
    fn test_usage() {
        let mut session = Session::new(
            net_id::PeerId::from("me"),
            Vec::new(),
            discovery::Timing::default(),
        );
        let room = net_id::RoomName::from("r");
        session.room = Some(discovery::Room {
            name: room.clone(),
            members: discovery::Members::new(),
        });
        let presence = discovery::Presence {
            addrs: vec![net_id::PeerAddr::from("/ip4/1")],
            updated_at: discovery::EpochSecs(5),
        };
        let members = BTreeMap::from([
            (net_id::PeerId::from("me"), presence.clone()),
            (net_id::PeerId::from("a"), presence),
        ]);
        session.lobby.insert(
            room,
            discovery::RoomRecord {
                capacity: 8,
                members,
            },
        );
        seed_candidates(&mut session);
        let seeded: Vec<_> = session.candidates.keys().cloned().collect();
        assert_eq!(seeded, vec![net_id::PeerId::from("a")]);
    }
}
