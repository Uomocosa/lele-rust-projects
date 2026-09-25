use bevy::prelude::Resource;
use derive_more::Deref;

use super::constants;

#[derive(Resource, Debug, Clone, PartialEq, Eq, Deref)]
pub struct Lobby(pub String);

impl Default for Lobby {
    fn default() -> Self {
        Self(constants::DEFAULT_LOBBY.to_string())
    }
}

impl Lobby {
    #[must_use]
    pub const fn new(name: String) -> Self {
        Self(name)
    }
}

#[cfg(test)]
mod tests {
    use super::Lobby;

    #[test]
    fn test_usage() {
        assert_eq!(&*Lobby::default(), "default");
        assert_eq!(&*Lobby::new("alpha".to_string()), "alpha");
    }
}
