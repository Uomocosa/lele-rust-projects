use std::time::Instant;

use crate::discovery;
use discovery::id::RoomName;
use discovery::link::NetLink;
use discovery::room_peers::Members;
use discovery::session::{Room, Session, broadcast_hello, dial_candidates, seed_from_catalogue};
use discovery::{Command, Event, Timing};

pub fn apply_command(
    session: &mut Session,
    link: &NetLink,
    command: Command,
    timing: &Timing,
    events: &tokio::sync::mpsc::UnboundedSender<Event>,
) {
    match command {
        Command::Create(room) | Command::Join(room) => {
            join(session, link, room, timing, events);
        }
        Command::Leave => leave(session, link, events),
    }
}

// needed helper: enters a room, says hello to every live link and dials the board seeds
fn join(
    session: &mut Session,
    link: &NetLink,
    room: RoomName,
    timing: &Timing,
    events: &tokio::sync::mpsc::UnboundedSender<Event>,
) {
    leave(session, link, events);
    session.room = Some(Room {
        name: room.clone(),
        members: Members::new(),
    });
    seed_from_catalogue(session);
    broadcast_hello(session, link);
    let redial = std::time::Duration::from_secs(timing.redial_secs);
    dial_candidates(session, link, redial, Instant::now());
    tracing::info!(target: "room_lobby", room = %room.as_str(), "discovery joined room");
    let _ = events.send(Event::Joined(room));
}

// needed helper: leaves the current room and tells every live link
fn leave(
    session: &mut Session,
    link: &NetLink,
    events: &tokio::sync::mpsc::UnboundedSender<Event>,
) {
    let Some(room) = session.room.take() else {
        return;
    };
    session.candidates.clear();
    session.last_dial.clear();
    broadcast_hello(session, link);
    let _ = events.send(Event::Left(room.name));
}

// no test_usage necessary
