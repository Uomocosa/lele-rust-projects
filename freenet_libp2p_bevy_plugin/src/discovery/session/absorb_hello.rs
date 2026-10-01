use crate::discovery;
use discovery::Event;
use discovery::id::{EpochSecs, Presence, RemotePeerId};
use discovery::link::NetLink;
use discovery::room_peers::{DiscoveryStatus, Member};
use discovery::session::{Hello, Session, add_candidates, send_hello};

pub fn absorb_hello(
    session: &mut Session,
    link: &NetLink,
    from: &RemotePeerId,
    data: &[u8],
    events: &tokio::sync::mpsc::UnboundedSender<Event>,
    now: EpochSecs,
) {
    let Ok(hello) = bincode::deserialize::<Hello>(data) else {
        return;
    };
    let my_room = session.room.as_ref().map(|room| room.name.clone());
    if hello.room.is_none() || hello.room != my_room {
        drop_member(session, from, events);
        return;
    }
    let mut peers = hello.peers;
    peers.push((from.clone(), hello.addrs.clone()));
    add_candidates(session, peers, now);
    if !session.connected.contains(from) {
        return;
    }
    let Some(room) = session.room.as_mut() else {
        return;
    };
    let member = Member {
        presence: Presence {
            addrs: hello.addrs,
            updated_at: now,
        },
        status: DiscoveryStatus::Connected,
    };
    let added = room.members.insert(from.clone(), member).is_none();
    if added {
        tracing::info!(
            target: "room_lobby",
            peer = %from.as_str(),
            room = %room.name.as_str(),
            members = room.members.len(),
            "discovery member joined"
        );
        let _ = events.send(Event::MembersChanged);
        send_hello(session, link, from);
    }
}

// needed helper: removes a peer that left our room or switched to another one
fn drop_member(
    session: &mut Session,
    peer: &RemotePeerId,
    events: &tokio::sync::mpsc::UnboundedSender<Event>,
) {
    let removed = session
        .room
        .as_mut()
        .and_then(|room| room.members.remove(peer))
        .is_some();
    if removed {
        tracing::info!(target: "room_lobby", peer = %peer.as_str(), "discovery member left");
        let _ = events.send(Event::MembersChanged);
    }
}

#[cfg(test)]
mod tests {
    use super::absorb_hello;
    use crate::discovery;
    use crate::p2p;
    use discovery::id::{EpochSecs, RemotePeerId, RoomName};
    use discovery::session::Hello;

    #[test]
    fn test_usage() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let (_, observed) = tokio::sync::watch::channel(None);
        let link = discovery::link::NetLink { tx, observed };
        let (events, _events_rx) = tokio::sync::mpsc::unbounded_channel();
        let mut session = discovery::session::Session::new(
            discovery::id::UniqueGameId::new(
                &discovery::id::GameName("g".to_string()),
                &discovery::id::GameToken("t".to_string()),
            ),
            RemotePeerId("me".to_string()),
            Vec::new(),
            8,
        );
        session.room = Some(discovery::session::Room {
            name: RoomName("r".to_string()),
            members: discovery::room_peers::Members::new(),
        });
        let peer = RemotePeerId("a".to_string());
        session.connected.insert(peer.clone());
        let hello = Hello {
            room: Some(RoomName("r".to_string())),
            addrs: vec!["/ip4/1".to_string()],
            peers: vec![(RemotePeerId("b".to_string()), vec!["/ip4/2".to_string()])],
        };
        let data = bincode::serialize(&hello).unwrap_or_default();
        absorb_hello(&mut session, &link, &peer, &data, &events, EpochSecs(1));
        let members = session.room.as_ref().map_or(0, |room| room.members.len());
        assert_eq!(members, 1);
        assert_eq!(session.candidates.len(), 2);
        assert!(matches!(
            rx.try_recv(),
            Ok(p2p::NetCommand::Exchange { .. })
        ));
        let left = bincode::serialize(&Hello::default()).unwrap_or_default();
        absorb_hello(&mut session, &link, &peer, &left, &events, EpochSecs(2));
        let members = session.room.as_ref().map_or(0, |room| room.members.len());
        assert_eq!(members, 0);
    }

    #[test]
    fn connected_before_known_becomes_member() {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let (_, observed) = tokio::sync::watch::channel(None);
        let link = discovery::link::NetLink { tx, observed };
        let (events, _events_rx) = tokio::sync::mpsc::unbounded_channel();
        let mut session = discovery::session::Session::new(
            discovery::id::UniqueGameId::new(
                &discovery::id::GameName("g".to_string()),
                &discovery::id::GameToken("t".to_string()),
            ),
            RemotePeerId("me".to_string()),
            Vec::new(),
            8,
        );
        session.room = Some(discovery::session::Room {
            name: RoomName("r".to_string()),
            members: discovery::room_peers::Members::new(),
        });
        let joiner = RemotePeerId("joiner".to_string());
        session.connected.insert(joiner.clone());
        let hello = Hello {
            room: Some(RoomName("r".to_string())),
            addrs: vec!["/ip4/1".to_string()],
            peers: Vec::new(),
        };
        let data = bincode::serialize(&hello).unwrap_or_default();
        absorb_hello(&mut session, &link, &joiner, &data, &events, EpochSecs(1));
        let status = session
            .room
            .as_ref()
            .and_then(|room| room.members.get(&joiner))
            .map(|member| member.status);
        assert_eq!(
            status,
            Some(discovery::room_peers::DiscoveryStatus::Connected)
        );
    }
}
