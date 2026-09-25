use bevy::prelude::Message;

use crate::discovery::basic::newtypes::RoomName;

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
