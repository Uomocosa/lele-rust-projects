use libp2p::kad::RecordKey;

#[must_use]
pub fn provider_key(room: &str) -> RecordKey {
    RecordKey::new(&format!("lobby/room/{room}"))
}

#[cfg(test)]
mod tests {
    use super::provider_key;

    #[test]
    fn test_usage() {
        let key = provider_key("room-a");
        assert_ne!(key.as_ref().len(), 0);
    }
}
