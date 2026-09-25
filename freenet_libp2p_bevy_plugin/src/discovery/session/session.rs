use std::time::Instant;

use super::super::id::RemotePeerId;
use super::super::id::unique_game_id::UniqueGameId;
use super::super::lobby_rooms::RoomCatalogue;
use super::Room;

pub struct Session {
    pub id: UniqueGameId,
    pub me: RemotePeerId,
    pub addrs: Vec<String>,
    pub capacity: u16,
    pub catalogue: RoomCatalogue,
    pub room: Option<Room>,
    pub last_republish: Option<Instant>,
    pub last_board: Option<Instant>,
}

impl Session {
    #[must_use]
    pub const fn new(
        id: UniqueGameId,
        me: RemotePeerId,
        addrs: Vec<String>,
        capacity: u16,
    ) -> Self {
        Self {
            id,
            me,
            addrs,
            capacity,
            catalogue: RoomCatalogue::new(),
            room: None,
            last_republish: None,
            last_board: None,
        }
    }
}
// no test_usage necessary
