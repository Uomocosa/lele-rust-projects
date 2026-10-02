use crate::discovery;

use discovery::Timing;
use discovery::id::{GameName, GameToken};

#[derive(Debug, Clone)]
pub struct Config {
    pub game_name: GameName,
    pub token: GameToken,
    pub timing: Timing,
    pub capacity: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            game_name: GameName("test".to_string()),
            token: GameToken("test".to_string()),
            timing: Timing::default(),
            capacity: 8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let config = Config::default();
        let id = discovery::id::UniqueGameId::new(&config.game_name, &config.token);
        assert_eq!(id.as_str(), "test/test");
        assert_eq!(config.capacity, 8);
    }
}
