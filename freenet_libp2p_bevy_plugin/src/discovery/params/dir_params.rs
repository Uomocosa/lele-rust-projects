use super::super::constants;

#[must_use]
pub fn dir_params(namespace: &str) -> Vec<u8> {
    bincode::serialize(&(
        namespace.to_string(),
        constants::DIRECTORY_LOBBY.to_string(),
    ))
    .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::dir_params;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let dir = dir_params("blackboard-v1");
        let decoded: (String, String) = bincode::deserialize(&dir).unwrap_or_default();
        assert_eq!(decoded.1, discovery::DIRECTORY_LOBBY);
        assert_ne!(dir_params("blackboard-v1"), dir_params("other"));
    }
}
