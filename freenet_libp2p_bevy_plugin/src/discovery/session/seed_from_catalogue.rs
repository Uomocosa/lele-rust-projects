use crate::discovery;
use discovery::session::Session;

pub fn seed_from_catalogue(session: &mut Session) {
    let Some(room) = &session.room else {
        return;
    };
    let Some(record) = session.catalogue.get(&room.name) else {
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
            .is_none_or(|known| *known.updated_at < *presence.updated_at);
        if newer {
            session.candidates.insert(peer, presence);
        }
    }
}

// no test_usage necessary
