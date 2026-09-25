use super::directory_state::DirectoryState;
use super::merge_directory_entry;

#[must_use]
pub fn merge_directory(mut base: DirectoryState, other: DirectoryState) -> DirectoryState {
    for (room, entry) in other {
        let merged = merge_directory_entry::merge_directory_entry(base.remove(&room), entry);
        base.insert(room, merged);
    }
    base
}

#[cfg(test)]
mod tests {
    use super::merge_directory;
    use crate::discovery;

    fn entry(updated_at: u64) -> discovery::directory::Entry {
        discovery::directory::Entry {
            params: vec![1],
            peer_id: "peer".to_string(),
            addrs: Vec::new(),
            updated_at,
        }
    }

    #[test]
    fn test_usage() {
        let mut base = discovery::directory::DirectoryState::new();
        base.insert("room-a".to_string(), entry(5));
        let mut other = discovery::directory::DirectoryState::new();
        other.insert("room-a".to_string(), entry(9));
        let merged = merge_directory(base, other);
        assert_eq!(merged.get("room-a").map(|e| e.updated_at), Some(9));
    }
}
