use bevy::prelude::Message;

use super::id::room_name::RoomName;

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum Event {
    CatalogueChanged,
    Joined(RoomName),
    Left(RoomName),
    MembersChanged,
}

#[cfg(test)]
mod tests {
    use super::Event;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let room = discovery::id::RoomName("room-a".to_string());
        assert_eq!(Event::Joined(room.clone()), Event::Joined(room));
        assert_ne!(Event::CatalogueChanged, Event::MembersChanged);
    }
}
