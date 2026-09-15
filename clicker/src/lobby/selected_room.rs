use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct SelectedRoom(pub Option<String>);

#[cfg(test)]
mod tests {
    use super::SelectedRoom;

    #[test]
    fn test_usage() {
        let mut selected = SelectedRoom::default();
        assert!(selected.is_none());
        *selected = Some("room-a".to_string());
        assert_eq!(*selected, Some("room-a".to_string()));
    }
}
