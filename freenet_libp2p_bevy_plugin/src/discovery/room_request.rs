use bevy::prelude::Message;

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum RoomRequest {
    Join(String),
    Leave,
}

#[cfg(test)]
mod tests {
    use super::RoomRequest;

    #[test]
    fn test_usage() {
        assert_eq!(
            RoomRequest::Join("room-a".to_string()),
            RoomRequest::Join("room-a".to_string())
        );
        assert_ne!(RoomRequest::Leave, RoomRequest::Join("room-a".to_string()));
    }
}
