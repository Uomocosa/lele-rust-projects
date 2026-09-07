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

#[cfg(test)]
mod tests {
    use super::Lobby;

    #[test]
    fn test_usage() {
        assert_eq!(&*Lobby::default(), "default");
        assert_eq!(&*Lobby("alpha".to_string()), "alpha");
    }
}
