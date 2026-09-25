use super::active_room::ActiveRoom;
use super::left_room::LeftRoom;

#[must_use]
pub fn is_joinable(left: &LeftRoom, active: &ActiveRoom, room: &str) -> bool {
    if Some(room) == left.as_deref() {
        return false;
    }
    if active.as_deref() == Some(room) {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::{ActiveRoom, LeftRoom, is_joinable};

    #[test]
    fn test_usage() {
        let left = LeftRoom(Some("room-left".to_string()));
        let active = ActiveRoom(Some("room-a".to_string()));
        assert!(!is_joinable(&left, &active, "room-left"));
        assert!(!is_joinable(&LeftRoom::default(), &active, "room-a"));
        assert!(is_joinable(
            &LeftRoom::default(),
            &ActiveRoom::default(),
            "room-b"
        ));
    }
}
