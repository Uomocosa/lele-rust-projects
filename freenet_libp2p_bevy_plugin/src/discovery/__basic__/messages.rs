use bevy::prelude::Message;

use crate::discovery;

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Create(discovery::RoomName),
    Join(discovery::RoomName),
    Leave,
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum Event {
    LobbyChanged,
    Joined(discovery::RoomName),
    Left(discovery::RoomName),
    MembersChanged,
}
