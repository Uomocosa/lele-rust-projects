use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct ActiveRoom(pub Option<String>);

#[cfg(test)]
mod tests {
    use super::ActiveRoom;

    #[test]
    fn test_usage() {
        let mut active = ActiveRoom::default();
        *active = Some("room-a".to_string());
        assert_eq!(*active, Some("room-a".to_string()));
    }
}
