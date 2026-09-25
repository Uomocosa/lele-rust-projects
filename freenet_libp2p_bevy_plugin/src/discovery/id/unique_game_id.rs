use derive_more::Deref;

use super::GameName;
use super::GameToken;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref)]
pub struct UniqueGameId(pub String);

impl UniqueGameId {
    #[must_use]
    pub fn new(name: &GameName, token: &GameToken) -> Self {
        Self(format!("{}/{}", name.as_str(), token.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::{GameName, GameToken, UniqueGameId};

    #[test]
    fn test_usage() {
        let game = GameName("chess".to_string());
        let token = GameToken("bobs-chess".to_string());
        assert_eq!(
            UniqueGameId::new(&game, &token).as_str(),
            "chess/bobs-chess"
        );
        assert_ne!(
            UniqueGameId::new(&game, &token),
            UniqueGameId::new(&game, &GameToken("alice-chess".to_string()))
        );
    }
}
