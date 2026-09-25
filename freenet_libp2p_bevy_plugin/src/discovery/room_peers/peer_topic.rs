use crate::discovery;

#[must_use]
pub fn peer_topic(id: &discovery::id::UniqueGameId, room: &discovery::id::RoomName) -> String {
    format!("{}/{}", id.as_str(), room.as_str())
}

#[cfg(test)]
mod tests {
    use super::peer_topic;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let id = discovery::id::UniqueGameId::new(
            &discovery::id::GameName("chess".to_string()),
            &discovery::id::GameToken("token".to_string()),
        );
        let room = discovery::id::RoomName("room-a".to_string());
        assert_eq!(peer_topic(&id, &room), "chess/token/room-a");
    }
}
