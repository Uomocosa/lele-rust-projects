use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use super::super::params::room_name::RoomName;

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct JoinPending(pub Option<RoomName>);

#[cfg(test)]
mod tests {
    use super::JoinPending;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut pending = JoinPending::default();
        *pending = Some(discovery::params::RoomName("room-a".to_string()));
        assert!(
            pending
                .as_ref()
                .is_some_and(|room| room.as_str() == "room-a")
        );
    }
}
