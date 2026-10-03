use crate::discovery;
use discovery::session::{Hello, Output, Session};

pub fn handle_hello(
    session: &mut Session,
    from: &discovery::PeerId,
    data: &[u8],
    now: discovery::EpochSecs,
) {
    let Ok(hello) = bincode::deserialize::<Hello>(data) else {
        return;
    };
    let my_room = session.room.as_ref().map(|room| room.name.clone());
    if hello.room.is_none() || hello.room != my_room {
        drop_member(session, from);
        return;
    }
    let mut peers = hello.peers;
    peers.push((from.clone(), hello.addrs.clone()));
    discovery::session::add_candidates(session, peers, now);
    if !session.connected.contains(from) {
        return;
    }
    let Some(room) = session.room.as_mut() else {
        return;
    };
    let member = discovery::Member {
        presence: discovery::Presence {
            addrs: hello.addrs,
            updated_at: now,
        },
        status: discovery::LinkStatus::Connected,
    };
    let added = room.members.insert(from.clone(), member).is_none();
    if !added {
        return;
    }
    tracing::info!(
        target: "room_lobby",
        peer = %from.as_str(),
        room = %room.name.as_str(),
        members = room.members.len(),
        "discovery member joined"
    );
    session
        .outputs
        .push(Output::Event(discovery::Event::MembersChanged));
    discovery::session::send_hello(session, from);
}

// needed helper: removes a peer that left our room or switched to another one
fn drop_member(session: &mut Session, peer: &discovery::PeerId) {
    let removed = session
        .room
        .as_mut()
        .and_then(|room| room.members.remove(peer))
        .is_some();
    if !removed {
        return;
    }
    tracing::info!(target: "room_lobby", peer = %peer.as_str(), "discovery member left");
    session
        .outputs
        .push(Output::Event(discovery::Event::MembersChanged));
}

#[cfg(test)]
mod tests {
    use super::handle_hello;
    use crate::discovery;
    use crate::p2p;
    use discovery::session::{Hello, Output, Session};

    fn session_in_room() -> Session {
        let mut session = Session::new(
            discovery::PeerId("me".to_string()),
            Vec::new(),
            discovery::Timing::default(),
        );
        session.room = Some(discovery::Room {
            name: discovery::RoomName("r".to_string()),
            members: discovery::Members::new(),
        });
        session
    }

    #[test]
    fn test_usage() {
        let mut session = session_in_room();
        let peer = discovery::PeerId("a".to_string());
        session.connected.insert(peer.clone());
        let hello = Hello {
            room: Some(discovery::RoomName("r".to_string())),
            addrs: vec!["/ip4/1".to_string()],
            peers: vec![(
                discovery::PeerId("b".to_string()),
                vec!["/ip4/2".to_string()],
            )],
        };
        let data = bincode::serialize(&hello).unwrap_or_default();
        handle_hello(&mut session, &peer, &data, discovery::EpochSecs(1));
        assert_eq!(
            session.room.as_ref().map_or(0, |room| room.members.len()),
            1
        );
        assert_eq!(session.candidates.len(), 2);
        assert!(matches!(
            std::mem::take(&mut session.outputs).as_slice(),
            [
                Output::Event(discovery::Event::MembersChanged),
                Output::Net(p2p::NetCommand::Exchange { .. }),
            ]
        ));
        let left = bincode::serialize(&Hello::default()).unwrap_or_default();
        handle_hello(&mut session, &peer, &left, discovery::EpochSecs(2));
        assert_eq!(
            session.room.as_ref().map_or(0, |room| room.members.len()),
            0
        );
    }

    #[test]
    fn test_connected_before_known_becomes_member() {
        let mut session = session_in_room();
        let joiner = discovery::PeerId("joiner".to_string());
        session.connected.insert(joiner.clone());
        let hello = Hello {
            room: Some(discovery::RoomName("r".to_string())),
            addrs: vec!["/ip4/1".to_string()],
            peers: Vec::new(),
        };
        let data = bincode::serialize(&hello).unwrap_or_default();
        handle_hello(&mut session, &joiner, &data, discovery::EpochSecs(1));
        let status = session
            .room
            .as_ref()
            .and_then(|room| room.members.get(&joiner))
            .map(|member| member.status);
        assert_eq!(status, Some(discovery::LinkStatus::Connected));
    }
}
