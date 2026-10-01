use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use crate::discovery;
use discovery::id::{Presence, RemotePeerId, UniqueGameId};
use discovery::lobby_rooms::RoomCatalogue;
use discovery::session::Room;

pub struct Session {
    pub id: UniqueGameId,
    pub me: RemotePeerId,
    pub addrs: Vec<String>,
    pub capacity: u16,
    pub catalogue: RoomCatalogue,
    pub room: Option<Room>,
    pub connected: BTreeSet<RemotePeerId>,
    pub candidates: BTreeMap<RemotePeerId, Presence>,
    pub last_dial: BTreeMap<RemotePeerId, Instant>,
    pub last_hello: Option<Instant>,
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
            connected: BTreeSet::new(),
            candidates: BTreeMap::new(),
            last_dial: BTreeMap::new(),
            last_hello: None,
        }
    }
}
// no test_usage necessary
