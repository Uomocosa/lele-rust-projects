use std::collections::BTreeMap;

use bevy::prelude::Resource;

use super::id::room_name::RoomName;
use super::id::room_record::RoomRecord;
use super::session::Room;

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct Multiplayer {
    pub catalogue: BTreeMap<RoomName, RoomRecord>,
    pub room: Option<Room>,
}

#[cfg(test)]
mod tests {
    use super::Multiplayer;

    #[test]
    fn test_usage() {
        let multiplayer = Multiplayer::default();
        assert!(multiplayer.catalogue.is_empty());
        assert!(multiplayer.room.is_none());
    }
}
