use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use super::super::params::room_name::RoomName;

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct ActiveRoom(pub Option<RoomName>);

#[cfg(test)]
mod tests {
    use super::ActiveRoom;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut active = ActiveRoom::default();
        *active = Some(discovery::params::RoomName("room-a".to_string()));
        assert!(
            active
                .as_ref()
                .is_some_and(|room| room.as_str() == "room-a")
        );
    }
}
