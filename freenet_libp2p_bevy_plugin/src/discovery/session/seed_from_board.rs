use crate::discovery;
use discovery::id::now_epoch;
use discovery::room_peers::merge_peers;
use discovery::session::Session;

pub fn seed_from_board(session: &mut Session) {
    let Some(room_name) = session.room.as_ref().map(|room| room.name.clone()) else {
        return;
    };
    let Some(record) = session.catalogue.get(&room_name) else {
        return;
    };
    let peers = record
        .members
        .iter()
        .map(|(peer, presence)| (peer.clone(), presence.addrs.clone()))
        .collect();
    if let Some(room) = session.room.as_mut() {
        merge_peers(&mut room.members, peers, now_epoch(), &session.me);
    }
}

// no test_usage necessary
