use super::directory_state::DirectoryState;
use super::entry::Entry;

#[must_use]
pub fn pick_room(state: &DirectoryState, since_secs: u64) -> Option<(String, Entry)> {
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
        let mut state = discovery::directory::DirectoryState::new();
        state.insert("room-old".to_string(), entry(100));
        state.insert("room-new".to_string(), entry(200));
        assert_eq!(
            pick_room(&state, 50).map(|(room, _)| room),
            Some("room-new".to_string())
        );
        assert!(pick_room(&state, 300).is_none());
    }
}
