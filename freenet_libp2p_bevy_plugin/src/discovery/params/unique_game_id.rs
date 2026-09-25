use derive_more::Deref;
use serde::{Deserialize, Serialize};

use super::game_name::GameName;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Deref)]
pub struct UniqueGameId(pub String);

impl UniqueGameId {
    #[must_use]
    pub fn new(name: &GameName, token: &str) -> Self {
        Self(format!("{}/{token}", name.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::{GameName, UniqueGameId};

    #[test]
    fn test_usage() {
        let game = GameName("chess".to_string());
        assert_eq!(
            UniqueGameId::new(&game, "a3f9c12e").as_str(),
            "chess/a3f9c12e"
        );
        assert_ne!(
            UniqueGameId::new(&game, "a3f9c12e"),
            UniqueGameId::new(&game, "77bd0102")
        );
    }
}
