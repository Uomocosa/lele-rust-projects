use super::super::constants;
use super::super::params::room_name::RoomName;
use super::super::params::unique_game_id::UniqueGameId;

#[must_use]
pub fn roster_topic(id: &UniqueGameId, room: &RoomName) -> String {
    format!(
        "{}/{}/{}",
        id.as_str(),
        room.as_str(),
        constants::ROSTER_TOPIC_SUFFIX
    )
}

#[cfg(test)]
mod tests {
    use super::roster_topic;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let id = discovery::params::UniqueGameId::new(
            &discovery::params::GameName("test".to_string()),
            "token",
        );
        let room = discovery::params::RoomName("room".to_string());
        assert_eq!(roster_topic(&id, &room), "test/token/room/roster");
    }
}
