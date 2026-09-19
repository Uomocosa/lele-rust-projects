use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct LeftRoom(pub Option<String>);

#[cfg(test)]
mod tests {
    use super::LeftRoom;

    #[test]
    fn test_usage() {
        let mut left = LeftRoom::default();
        assert!(left.is_none());
        *left = Some("room-a".to_string());
        assert_eq!(*left, Some("room-a".to_string()));
    }
}
