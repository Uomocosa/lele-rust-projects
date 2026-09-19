#[must_use]
pub fn key_for_peer(name: &str) -> [u8; 32] {
    match name {
        "peer-1" => [1u8; 32],
        "peer-2" => [2u8; 32],
        _ => [3u8; 32],
    }
}

#[cfg(test)]
mod tests {
    use super::key_for_peer;

    #[test]
    fn test_usage() {
        assert_eq!(key_for_peer("peer-1"), [1u8; 32]);
        assert_eq!(key_for_peer("peer-2"), [2u8; 32]);
        assert_eq!(key_for_peer("peer-9"), [3u8; 32]);
    }
}
