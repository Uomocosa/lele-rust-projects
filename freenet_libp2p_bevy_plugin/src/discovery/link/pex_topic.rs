use super::super::constants;
use super::super::params::unique_game_id::UniqueGameId;

#[must_use]
pub fn pex_topic(id: &UniqueGameId) -> String {
    format!("{}/{}", id.as_str(), constants::PEX_TOPIC_SUFFIX)
}

#[cfg(test)]
mod tests {
    use super::pex_topic;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let id = discovery::params::UniqueGameId::new(
            &discovery::params::GameName("test".to_string()),
            "token",
        );
        assert_eq!(pex_topic(&id), "test/token/pex");
    }
}
