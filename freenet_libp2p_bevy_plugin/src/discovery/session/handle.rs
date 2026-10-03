use crate::discovery;
use discovery::session::{Input, Session};

pub fn handle(session: &mut Session, input: Input, now: discovery::EpochSecs) {
    match input {
        Input::Command(command) => discovery::session::handle_command(session, command, now),
        Input::Net(event) => discovery::session::handle_net_event(session, event, now),
        Input::Lobby(lobby) => {
            discovery::session::handle_lobby(session, lobby, now);
        }
        Input::Tick => discovery::session::tick(session, now),
    }
}

#[cfg(test)]
mod tests {
    use super::handle;
    use crate::discovery;
    use discovery::session::{Input, Output, Session};

    #[test]
    fn test_usage() {
        let mut session = Session::new(
            discovery::PeerId("me".to_string()),
            Vec::new(),
            discovery::Timing::default(),
        );
        let room = discovery::RoomName("r".to_string());
        handle(
            &mut session,
            Input::Command(discovery::Command::Join(room.clone())),
            discovery::EpochSecs(1),
        );
        assert_eq!(
            std::mem::take(&mut session.outputs),
            vec![Output::Event(discovery::Event::Joined(room))]
        );
    }
}
