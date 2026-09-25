use super::super::constants;
use super::super::params::unique_game_id::UniqueGameId;

#[must_use]
pub fn is_roster_topic(id: &UniqueGameId, topic: &str) -> bool {
    topic.starts_with(&format!("{}/", id.as_str()))
        && topic.ends_with(&format!("/{}", constants::ROSTER_TOPIC_SUFFIX))
}

#[cfg(test)]
mod tests {
    use super::is_roster_topic;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let id = discovery::params::UniqueGameId::new(
            &discovery::params::GameName("test".to_string()),
            "token",
        );
        assert!(is_roster_topic(&id, "test/token/room/roster"));
        assert!(!is_roster_topic(&id, "test/token/pex"));
    }
}
