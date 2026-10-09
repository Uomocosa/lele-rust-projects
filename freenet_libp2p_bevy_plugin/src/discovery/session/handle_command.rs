use crate::discovery;
use crate::net_id;
use discovery::session::{Output, Session};

pub fn handle_command(
    session: &mut Session,
    command: discovery::Command,
    now: discovery::EpochSecs,
) {
    match command {
        discovery::Command::Create(room) | discovery::Command::Join(room) => {
            join(session, room, now);
        }
        discovery::Command::Leave => leave(session),
    }
}

// needed helper: enters a room, says hello to every live link and dials the lobby seeds
fn join(session: &mut Session, room: net_id::RoomName, now: discovery::EpochSecs) {
    leave(session);
    session.room = Some(discovery::Room {
        name: room.clone(),
        members: discovery::Members::new(),
    });
    discovery::session::seed_candidates(session);
    discovery::session::broadcast_hello(session);
    discovery::session::dial_candidates(session, now);
    tracing::info!(target: "room_lobby", room = %room.as_str(), "discovery joined room");
    session
        .outputs
        .push(Output::Event(discovery::Event::Joined(room)));
}

// needed helper: leaves the current room and tells every live link
fn leave(session: &mut Session) {
    let Some(room) = session.room.take() else {
        return;
    };
    session.candidates.clear();
    session.last_dial.clear();
    discovery::session::broadcast_hello(session);
    session
        .outputs
        .push(Output::Event(discovery::Event::Left(room.name)));
}

#[cfg(test)]
mod tests {
    use super::handle_command;
    use crate::discovery;
    use crate::net_id;
    use crate::p2p;
    use discovery::session::{Output, Session};

    #[test]
    fn test_usage() {
        let mut session = Session::new(
            net_id::PeerId::from("me"),
            Vec::new(),
            discovery::Timing::default(),
        );
        session.connected.insert(net_id::PeerId::from("a"));
        let room = net_id::RoomName::from("r");
        handle_command(
            &mut session,
            discovery::Command::Create(room),
            discovery::EpochSecs(1),
        );
        assert!(session.room.is_some());
        handle_command(
            &mut session,
            discovery::Command::Leave,
            discovery::EpochSecs(2),
        );
        assert!(session.room.is_none());
        let outputs = std::mem::take(&mut session.outputs);
        assert!(matches!(
            outputs.as_slice(),
            [
                Output::Net(p2p::NetCommand::Exchange { .. }),
                Output::Event(discovery::Event::Joined(_)),
                Output::Net(p2p::NetCommand::Exchange { .. }),
                Output::Event(discovery::Event::Left(_)),
            ]
        ));
    }
}
