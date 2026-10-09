use crate::discovery;
use crate::net_id;
use discovery::state_machine::{Output, State};

pub fn handle_command(state: &mut State, command: discovery::Command, now: discovery::EpochSecs) {
    match command {
        discovery::Command::Create(room) | discovery::Command::Join(room) => {
            join(state, room, now);
        }
        discovery::Command::Leave => leave(state),
    }
}

// needed helper: enters a room, says hello to every live link and dials the lobby seeds
fn join(state: &mut State, room: net_id::RoomName, now: discovery::EpochSecs) {
    leave(state);
    state.room = Some(discovery::Room {
        name: room.clone(),
        members: discovery::Members::new(),
    });
    discovery::state_machine::seed_candidates(state);
    discovery::state_machine::broadcast_hello(state);
    discovery::state_machine::dial_candidates(state, now);
    tracing::info!(target: "room_lobby", room = %room.as_str(), "discovery joined room");
    state
        .outputs
        .push(Output::Event(discovery::Event::Joined(room)));
}

// needed helper: leaves the current room and tells every live link
fn leave(state: &mut State) {
    let Some(room) = state.room.take() else {
        return;
    };
    state.candidates.clear();
    state.last_dial.clear();
    discovery::state_machine::broadcast_hello(state);
    state
        .outputs
        .push(Output::Event(discovery::Event::Left(room.name)));
}

#[cfg(test)]
mod tests {
    use super::handle_command;
    use crate::discovery;
    use crate::net_id;
    use crate::p2p;
    use discovery::state_machine::{Output, State};

    #[test]
    fn test_usage() {
        let mut state = State::new(
            net_id::Peer {
                id: net_id::PeerId::from("me"),
                addrs: Vec::new(),
            },
            discovery::Timing::default(),
        );
        state.connected.insert(net_id::PeerId::from("a"));
        let room = net_id::RoomName::from("r");
        handle_command(
            &mut state,
            discovery::Command::Create(room),
            discovery::EpochSecs(1),
        );
        assert!(state.room.is_some());
        handle_command(
            &mut state,
            discovery::Command::Leave,
            discovery::EpochSecs(2),
        );
        assert!(state.room.is_none());
        let outputs = std::mem::take(&mut state.outputs);
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
