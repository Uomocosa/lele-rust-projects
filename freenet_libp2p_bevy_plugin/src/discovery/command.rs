use bevy::prelude::Message;

use super::id::room_name::RoomName;

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Create(RoomName),
    Join(RoomName),
    Leave,
}

#[cfg(test)]
mod tests {
    use super::Command;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let room = discovery::id::RoomName("room-a".to_string());
        assert_eq!(Command::Join(room.clone()), Command::Join(room));
        assert_ne!(
            Command::Leave,
            Command::Join(discovery::id::RoomName("room-a".to_string()))
        );
    }
}
