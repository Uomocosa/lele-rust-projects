use std::time::Instant;

use crate::discovery;
use discovery::id::RemotePeerId;
use discovery::id::UniqueGameId;
use discovery::lobby_rooms::RoomCatalogue;
use discovery::session::Room;

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
