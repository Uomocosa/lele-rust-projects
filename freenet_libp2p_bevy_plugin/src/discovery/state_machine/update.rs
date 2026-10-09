use crate::discovery;
use discovery::state_machine::{Input, State};

pub fn update(state: &mut State, input: Input, now: discovery::EpochSecs) {
    match input {
        Input::Command(command) => discovery::state_machine::handle_command(state, command, now),
        Input::Net(event) => discovery::state_machine::handle_net_event(state, event, now),
        Input::Lobby(lobby) => {
            discovery::state_machine::handle_lobby(state, lobby, now);
        }
        Input::Tick => discovery::state_machine::tick(state, now),
    }
}

#[cfg(test)]
mod tests {
    use super::update;
    use crate::discovery;
    use crate::net_id;
    use discovery::state_machine::{Input, Output, State};

    #[test]
    fn test_usage() {
        let mut state = State::new(
            net_id::Peer {
                id: net_id::PeerId::from("me"),
                addrs: Vec::new(),
            },
            discovery::Timing::default(),
        );
        let room = net_id::RoomName::from("r");
        update(
            &mut state,
            Input::Command(discovery::Command::Join(room.clone())),
            discovery::EpochSecs(1),
        );
        assert_eq!(
            std::mem::take(&mut state.outputs),
            vec![Output::Event(discovery::Event::Joined(room))]
        );
    }
}
