use super::super::id::room_name::RoomName;
use super::super::room_peers::Members;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Room {
    pub name: RoomName,
    pub members: Members,
}

#[cfg(test)]
mod tests {
    use super::Room;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let room = Room {
            name: discovery::id::RoomName("room-a".to_string()),
            members: discovery::room_peers::Members::new(),
        };
        assert_eq!(room.name.as_str(), "room-a");
        assert!(room.members.is_empty());
    }
}
