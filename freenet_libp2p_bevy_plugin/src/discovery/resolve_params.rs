use super::contract_params;

#[must_use]
pub fn resolve_params(namespace: &str, lobby: &str, override_hex: Option<&str>) -> Vec<u8> {
    override_hex
        .filter(|v| !v.is_empty())
        .and_then(|v| hex::decode(v.trim_start_matches("0x")).ok())
        .unwrap_or_else(|| contract_params::contract_params(namespace, lobby))
}

#[cfg(test)]
mod tests {
    use super::resolve_params;

    #[test]
    fn test_usage() {
        let fallback = resolve_params("blackboard-v1", "alpha", None);
        let explicit = hex::encode(&fallback);
        assert_eq!(
            resolve_params("blackboard-v1", "alpha", Some(&explicit)),
            fallback
        );
        assert_eq!(
            resolve_params("blackboard-v1", "alpha", Some("not-hex!!")),
            fallback
        );
    }
}
