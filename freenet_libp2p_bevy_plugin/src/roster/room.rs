use bevy::prelude::Resource;
use derive_more::Deref;

use crate::roster::basic::constants;

#[derive(Resource, Debug, Clone, PartialEq, Eq, Deref)]
pub struct Room(pub String);

impl Default for Room {
    fn default() -> Self {
        Self(constants::DEFAULT_ROOM.to_string())
    }
}

impl Room {
    #[must_use]
    pub const fn new(name: String) -> Self {
        Self(name)
    }
}

#[cfg(test)]
mod tests {
    use super::Room;

    #[test]
    fn test_usage() {
        assert_eq!(&*Room::default(), "default");
        assert_eq!(&*Room::new("alpha".to_string()), "alpha");
    }
}
