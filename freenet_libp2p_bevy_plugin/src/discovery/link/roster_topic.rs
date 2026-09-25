use super::super::constants;

#[must_use]
pub fn roster_topic(namespace: &str, room: &str) -> String {
    format!("{namespace}/{room}/{}", constants::ROSTER_TOPIC_SUFFIX)
}

#[cfg(test)]
mod tests {
    use super::roster_topic;

    #[test]
    fn test_usage() {
        assert_eq!(roster_topic("ns", "room"), "ns/room/roster");
    }
}
