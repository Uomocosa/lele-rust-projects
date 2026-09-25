use bevy::prelude::Message;

use super::super::params::room_name::RoomName;

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum RoomRequest {
    Join(RoomName),
    Leave,
}

#[cfg(test)]
mod tests {
    use super::RoomRequest;
    use crate::discovery;

    #[test]
    fn test_usage() {
        assert_eq!(
            RoomRequest::Join(discovery::params::RoomName("room-a".to_string())),
            RoomRequest::Join(discovery::params::RoomName("room-a".to_string()))
        );
        assert_ne!(
            RoomRequest::Leave,
            RoomRequest::Join(discovery::params::RoomName("room-a".to_string()))
        );
    }
}
