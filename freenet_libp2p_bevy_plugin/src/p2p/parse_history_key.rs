#[must_use]
pub fn parse_history_key(key: &str) -> Option<(String, u64)> {
    let rest = key.strip_prefix("lobby/history/")?;
    let (room, chunk) = rest.rsplit_once('/')?;
    Some((room.to_string(), chunk.parse().ok()?))
}

#[cfg(test)]
mod tests {
    use super::parse_history_key;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let key = p2p::history_key("room-a", 7);
        let key = String::from_utf8_lossy(key.as_ref()).to_string();
        assert_eq!(parse_history_key(&key), Some(("room-a".to_string(), 7)));
    }

    #[test]
    fn test_usage_room_with_slash_round_trips() {
        let key = p2p::history_key("team/blue", 3);
        let key = String::from_utf8_lossy(key.as_ref()).to_string();
        assert_eq!(parse_history_key(&key), Some(("team/blue".to_string(), 3)));
    }

    #[test]
    fn test_usage_rejects_non_history_keys() {
        assert_eq!(parse_history_key("lobby/room/room-a"), None);
        assert_eq!(parse_history_key("lobby/history/room-a/xx"), None);
        assert_eq!(parse_history_key("a/b/c/d"), None);
    }
}
