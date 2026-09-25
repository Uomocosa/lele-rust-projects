use bevy::prelude::Message;

use super::newtypes::RoomName;

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Create(RoomName),
    Join(RoomName),
    Leave,
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum Event {
    CatalogueChanged,
    Joined(RoomName),
    Left(RoomName),
    MembersChanged,
}

#[cfg(test)]
mod tests {
    use super::{Command, Event};
    use crate::discovery;

    #[test]
    fn test_usage() {
        let room = discovery::id::RoomName("room-a".to_string());
        assert_eq!(Command::Join(room.clone()), Command::Join(room.clone()));
        assert_ne!(Command::Leave, Command::Join(room.clone()));
        assert_eq!(Event::Joined(room.clone()), Event::Joined(room));
        assert_ne!(Event::CatalogueChanged, Event::MembersChanged);
    }
}
