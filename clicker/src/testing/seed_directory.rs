use crate::discovery;

#[must_use]
pub fn seed_directory(room: &str, updated_at: u64) -> discovery::DirectoryState {
    let mut state = discovery::DirectoryState::new();
    state.insert(
        room.to_string(),
        discovery::DirectoryEntry {
            params: vec![1, 2, 3],
            peer_id: format!("peer-{room}"),
            addrs: vec![format!("/ip4/127.0.0.1/tcp/{updated_at}")],
            updated_at,
        },
    );
    state
}

#[cfg(test)]
mod tests {
    use super::seed_directory;

    #[test]
    fn test_usage() {
        assert_eq!(seed_directory("room-1", 100).len(), 1);
    }
}
