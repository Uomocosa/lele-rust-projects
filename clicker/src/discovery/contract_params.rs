#[must_use]
pub fn contract_params(namespace: &str, lobby: &str) -> Vec<u8> {
    bincode::serialize(&(namespace.to_string(), lobby.to_string())).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::contract_params;

    #[test]
    fn test_usage() {
        let params = contract_params("blackboard-v1", "alpha");
        assert_ne!(params, Vec::<u8>::new());
        let decoded: (String, String) = bincode::deserialize(&params).unwrap_or_default();
        assert_eq!(decoded, ("blackboard-v1".to_string(), "alpha".to_string()));
        assert_ne!(
            contract_params("blackboard-v1", "alpha"),
            contract_params("blackboard-v1", "beta")
        );
    }
}
