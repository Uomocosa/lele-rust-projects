use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use super::super::params::room_name::RoomName;

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct SelectedRoom(pub Option<RoomName>);

#[cfg(test)]
mod tests {
    use super::SelectedRoom;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut selected = SelectedRoom::default();
        *selected = Some(discovery::params::RoomName("room-a".to_string()));
        assert!(
            selected
                .as_ref()
                .is_some_and(|room| room.as_str() == "room-a")
        );
    }
}
