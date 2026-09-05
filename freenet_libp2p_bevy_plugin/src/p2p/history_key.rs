use libp2p::kad::RecordKey;

#[must_use]
pub fn history_key(lobby: &str, chunk: u64) -> RecordKey {
    RecordKey::new(&format!("blackboard/history/{lobby}/{chunk:08}"))
}

#[cfg(test)]
mod tests {
    use super::history_key;

    #[test]
    fn test_usage() {
        let key = history_key("lobby-a", 0);
        assert_ne!(key.as_ref().len(), 0);
    }
}
