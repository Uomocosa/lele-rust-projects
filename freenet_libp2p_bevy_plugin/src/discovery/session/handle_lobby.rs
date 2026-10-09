use crate::discovery;
use discovery::session::{Output, Session};

pub fn handle_lobby(session: &mut Session, lobby: discovery::Lobby, now: discovery::EpochSecs) {
    let ttl_secs = session.timing.presence_ttl_secs;
    let lobby = discovery::session::live_lobby(lobby, now, ttl_secs);
    if lobby == session.lobby {
        return;
    }
    session.lobby = lobby;
    session
        .outputs
        .push(Output::Event(discovery::Event::LobbyChanged));
    discovery::session::seed_candidates(session);
}

#[cfg(test)]
mod tests {
    use crate::net_id;
    use std::collections::BTreeMap;

    use super::handle_lobby;
    use crate::discovery;
    use discovery::session::{Output, Session};

    #[test]
    fn test_usage() {
        let mut session = Session::new(
            net_id::PeerId("me".to_string()),
            Vec::new(),
            discovery::Timing::default(),
        );
        session.room = Some(discovery::Room {
            name: net_id::RoomName("r".to_string()),
            members: discovery::Members::new(),
        });
        let mut members = BTreeMap::new();
        for peer in ["me", "a"] {
            members.insert(
                net_id::PeerId(peer.to_string()),
                discovery::Presence {
                    addrs: vec![net_id::PeerAddr("/ip4/1".to_string())],
                    updated_at: discovery::EpochSecs(3),
                },
            );
        }
        let mut lobby = discovery::Lobby::new();
        lobby.insert(
            net_id::RoomName("r".to_string()),
            discovery::RoomRecord {
                capacity: 8,
                members,
            },
        );
        handle_lobby(&mut session, lobby.clone(), discovery::EpochSecs(10));
        assert_eq!(session.candidates.len(), 1);
        assert_eq!(
            std::mem::take(&mut session.outputs),
            vec![Output::Event(discovery::Event::LobbyChanged)]
        );
        handle_lobby(&mut session, lobby, discovery::EpochSecs(10));
        assert_eq!(std::mem::take(&mut session.outputs), Vec::new());
    }
}
