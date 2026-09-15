use crate::discovery;

#[must_use]
pub fn dir_params(namespace: &str) -> Vec<u8> {
    bincode::serialize(&(
        namespace.to_string(),
        discovery::DIRECTORY_LOBBY.to_string(),
    ))
    .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::dir_params;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let params = dir_params("blackboard-v1");
        assert_ne!(params, Vec::<u8>::new());
        let decoded: (String, String) = bincode::deserialize(&params).unwrap_or_default();
        assert_eq!(decoded.1, discovery::DIRECTORY_LOBBY);
        assert_ne!(dir_params("blackboard-v1"), dir_params("other-ns"));
    }
}
