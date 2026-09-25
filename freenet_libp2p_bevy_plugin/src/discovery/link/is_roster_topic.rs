use super::super::constants;

#[must_use]
pub fn is_roster_topic(namespace: &str, topic: &str) -> bool {
    topic.starts_with(&format!("{namespace}/"))
        && topic.ends_with(&format!("/{}", constants::ROSTER_TOPIC_SUFFIX))
}

#[cfg(test)]
mod tests {
    use super::is_roster_topic;

    #[test]
    fn test_usage() {
        assert!(is_roster_topic("ns", "ns/room/roster"));
        assert!(!is_roster_topic("ns", "ns/pex"));
    }
}
