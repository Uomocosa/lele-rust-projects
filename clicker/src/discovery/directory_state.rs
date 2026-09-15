use std::collections::BTreeMap;

use crate::discovery;

pub type DirectoryState = BTreeMap<String, discovery::DirectoryEntry>;

#[cfg(test)]
mod tests {
    use super::DirectoryState;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut directory = DirectoryState::new();
        directory.insert(
            "room-20240101-120000".to_string(),
            discovery::DirectoryEntry {
                params: vec![1],
                peer_id: "peer".to_string(),
                addrs: Vec::new(),
                updated_at: 1,
            },
        );
        assert_eq!(directory.len(), 1);
    }
}
