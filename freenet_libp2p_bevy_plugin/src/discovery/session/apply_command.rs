use crate::discovery;
use crate::p2p;
use discovery::id::now_epoch;
use discovery::link::NetLink;
use discovery::room_peers::{Members, merge_peers, peer_topic};
use discovery::session::announce::announce;
use discovery::session::dial_known::dial_known;
use discovery::session::{Room, Session};
use discovery::{Command, Event};

pub fn apply_command(
    session: &mut Session,
    link: &NetLink,
    command: Command,
    events: &tokio::sync::mpsc::UnboundedSender<Event>,
) {
    match command {
        Command::Create(room) | Command::Join(room) => join(session, link, room, events),
        Command::Leave => {
            if let Some(room) = session.room.take() {
                let _ = events.send(Event::Left(room.name));
            }
        }
    }
}

// needed helper: seeds members from the catalogue, subscribes, announces and dials
fn join(
    session: &mut Session,
    link: &NetLink,
    room: discovery::id::RoomName,
    events: &tokio::sync::mpsc::UnboundedSender<Event>,
) {
    let mut members = Members::new();
    if let Some(record) = session.catalogue.get(&room) {
        let peers = record
            .members
            .iter()
            .map(|(peer, presence)| (peer.clone(), presence.addrs.clone()))
            .collect();
        merge_peers(&mut members, peers, now_epoch(), &session.me);
    }
    session.room = Some(Room {
        name: room.clone(),
        members,
    });
    link.tx
        .send(p2p::NetCommand::Subscribe {
            topic: peer_topic(&session.id, &room),
        })
        .ok();
    announce(session, link);
    dial_known(session, link);
    let _ = events.send(Event::Joined(room));
}

// no test_usage necessary
