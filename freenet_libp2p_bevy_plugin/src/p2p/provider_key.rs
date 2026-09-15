use libp2p::kad::RecordKey;

#[must_use]
pub fn provider_key(lobby: &str) -> RecordKey {
    RecordKey::new(&format!("blackboard/lobby/{lobby}"))
}

#[cfg(test)]
mod tests {
    use super::provider_key;

    #[test]
    fn test_usage() {
        let key = provider_key("lobby-a");
        assert_ne!(key.as_ref().len(), 0);
    }
}
