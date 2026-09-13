#[must_use]
pub fn fresh_params(namespace: &str, lobby: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let bytes =
        bincode::serialize(&(namespace.to_string(), lobby.to_string(), nanos)).unwrap_or_default();
    hex::encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::fresh_params;

    #[test]
    fn test_usage() {
        let first = fresh_params("blackboard-v1", "alpha");
        assert!(!first.is_empty());
        let decoded: (String, String, u128) =
            bincode::deserialize(&hex::decode(&first).unwrap_or_default()).unwrap_or_default();
        assert_eq!(decoded.0, "blackboard-v1");
        assert_eq!(decoded.1, "alpha");
    }
}
