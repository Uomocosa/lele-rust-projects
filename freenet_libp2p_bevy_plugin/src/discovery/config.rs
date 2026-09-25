use crate::p2p;

use super::params::discovery_timing::DiscoveryTiming;
use super::params::game_name::GameName;
use super::params::unique_game_id::UniqueGameId;

pub struct Config {
    pub id: UniqueGameId,
    pub timing: DiscoveryTiming,
    pub transport: p2p::TransportMode,
}

impl Default for Config {
    fn default() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        let token = format!("{nanos:x}-{}", std::process::id());
        Self {
            id: UniqueGameId::new(&GameName("test".to_string()), &token),
            timing: DiscoveryTiming::default(),
            transport: p2p::TransportMode::Both,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn test_usage() {
        let config = Config::default();
        assert_eq!(config.timing.tick_secs, 5);
        assert!(config.id.as_str().starts_with("test/"));
    }
}
