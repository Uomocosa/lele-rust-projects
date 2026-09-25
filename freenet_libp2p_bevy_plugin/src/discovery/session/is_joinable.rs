use super::super::params::room_name::RoomName;
use super::active_room::ActiveRoom;
use super::left_room::LeftRoom;

#[must_use]
pub fn is_joinable(left: &LeftRoom, active: &ActiveRoom, room: &RoomName) -> bool {
    if left.as_ref() == Some(room) {
        return false;
    }
    if active.as_ref() == Some(room) {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::{ActiveRoom, LeftRoom, is_joinable};
    use crate::discovery;

    #[test]
    fn test_usage() {
        let left = LeftRoom(Some(discovery::params::RoomName("room-left".to_string())));
        let active = ActiveRoom(Some(discovery::params::RoomName("room-a".to_string())));
        assert!(!is_joinable(
            &left,
            &active,
            &discovery::params::RoomName("room-left".to_string())
        ));
        assert!(!is_joinable(
            &LeftRoom::default(),
            &active,
            &discovery::params::RoomName("room-a".to_string())
        ));
        assert!(is_joinable(
            &LeftRoom::default(),
            &ActiveRoom::default(),
            &discovery::params::RoomName("room-b".to_string())
        ));
    }
}
