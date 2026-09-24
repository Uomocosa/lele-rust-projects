use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct JoinPending(pub Option<String>);

#[cfg(test)]
mod tests {
    use super::JoinPending;

    #[test]
    fn test_usage() {
        let mut pending = JoinPending::default();
        *pending = Some("room-a".to_string());
        assert_eq!(*pending, Some("room-a".to_string()));
    }
}
