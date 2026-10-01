use crate::discovery;
use discovery::Event;
use discovery::lobby_rooms::RoomCatalogue;
use discovery::session::{Session, seed_from_catalogue};

pub fn absorb_board(
    session: &mut Session,
    board: RoomCatalogue,
    events: &tokio::sync::mpsc::UnboundedSender<Event>,
) {
    if board == session.catalogue {
        return;
    }
    session.catalogue = board;
    let _ = events.send(Event::CatalogueChanged);
    seed_from_catalogue(session);
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::absorb_board;
    use crate::discovery;
    use discovery::id::{EpochSecs, Presence, RemotePeerId, RoomName, RoomRecord};

    #[test]
    fn test_usage() {
        let (events, mut events_rx) = tokio::sync::mpsc::unbounded_channel();
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
        let mut members = BTreeMap::new();
        for peer in ["me", "a"] {
            members.insert(
                RemotePeerId(peer.to_string()),
                Presence {
                    addrs: vec!["/ip4/1".to_string()],
                    updated_at: EpochSecs(3),
                },
            );
        }
        let mut board = BTreeMap::new();
        board.insert(
            RoomName("r".to_string()),
            RoomRecord {
                capacity: 8,
                members,
            },
        );
        absorb_board(&mut session, board, &events);
        assert_eq!(session.candidates.len(), 1);
        assert_eq!(events_rx.try_recv(), Ok(discovery::Event::CatalogueChanged));
    }
}
