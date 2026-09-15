use crate::discovery;

#[must_use]
pub fn pick_room(
    state: &discovery::DirectoryState,
    since_secs: u64,
) -> Option<(String, discovery::DirectoryEntry)> {
    state
        .iter()
        .filter(|(_, entry)| entry.updated_at > since_secs)
        .max_by_key(|(_, entry)| entry.updated_at)
        .map(|(room, entry)| (room.clone(), entry.clone()))
}

#[cfg(test)]
mod tests {
    use super::pick_room;
    use crate::discovery;

    fn entry(updated_at: u64) -> discovery::DirectoryEntry {
        discovery::DirectoryEntry {
            params: vec![1],
            peer_id: "peer".to_string(),
            addrs: Vec::new(),
            updated_at,
        }
    }

    #[test]
    fn test_usage() {
        let mut state = discovery::DirectoryState::new();
        state.insert("room-old".to_string(), entry(100));
        state.insert("room-new".to_string(), entry(200));
        assert_eq!(
            pick_room(&state, 50).map(|(room, _)| room),
            Some("room-new".to_string())
        );
        assert!(pick_room(&state, 200).is_none());
        assert!(pick_room(&state, 300).is_none());
        assert_eq!(
            pick_room(&state, 150).map(|(room, _)| room),
            Some("room-new".to_string())
        );
    }
}
