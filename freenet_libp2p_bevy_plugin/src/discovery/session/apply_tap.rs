use crate::discovery;
use crate::p2p;
use discovery::Event;
use discovery::id::{RemotePeerId, now_epoch};
use discovery::link::NetLink;
use discovery::room_peers::{DiscoveryStatus, MeshMessage, merge_peers, peer_topic};
use discovery::session::Session;
use discovery::session::announce::announce;
use discovery::session::dial_known::dial_known;

pub fn apply_tap(
    session: &mut Session,
    link: &NetLink,
    event: p2p::TapEvent,
    events: &tokio::sync::mpsc::UnboundedSender<Event>,
) {
    match event {
        p2p::TapEvent::PeerConnected(peer) => {
            mark(session, &RemotePeerId(peer), DiscoveryStatus::Connected);
        }
        p2p::TapEvent::PeerDisconnected(peer) => {
            mark(session, &RemotePeerId(peer), DiscoveryStatus::Known);
        }
        p2p::TapEvent::Gossip { topic, data, .. } => {
            absorb_gossip(session, link, &topic, &data, events);
        }
        _ => {}
    }
}

// needed helper: updates the live connection status of a known member
fn mark(session: &mut Session, peer: &RemotePeerId, status: DiscoveryStatus) {
    if let Some(room) = session.room.as_mut()
        && let Some(member) = room.members.get_mut(peer)
    {
        member.status = status;
    }
}

// needed helper: merges a gossip member list and replies with our own view
fn absorb_gossip(
    session: &mut Session,
    link: &NetLink,
    topic: &str,
    data: &[u8],
    events: &tokio::sync::mpsc::UnboundedSender<Event>,
) {
    let Some(room) = session.room.as_ref() else {
        return;
    };
    if topic != peer_topic(&session.id, &room.name) {
        return;
    }
    let Ok(message) = bincode::deserialize::<MeshMessage>(data) else {
        return;
    };
    let mut members = room.members.clone();
    let incoming = message.iter().cloned().collect();
    merge_peers(&mut members, incoming, now_epoch(), &session.me);
    if let Some(room) = session.room.as_mut() {
        room.members = members;
    }
    let _ = events.send(Event::MembersChanged);
    dial_known(session, link);
    announce(session, link);
}

// no test_usage necessary
