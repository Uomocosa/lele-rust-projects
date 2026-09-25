use std::collections::BTreeMap;
use std::sync::Mutex;

use bevy::prelude::Resource;
use derive_more::Deref;

use crate::discovery::basic::messages::{Command, Event};
use crate::discovery::basic::newtypes::RoomName;
use crate::discovery::basic::structs::{Room, RoomRecord};

#[derive(Resource, Debug, Deref)]
pub struct CommandSender(pub tokio::sync::mpsc::UnboundedSender<Command>);

#[derive(Resource, Debug, Deref)]
pub struct EventFeed(pub Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<Event>>>);

#[derive(Resource, Debug, Deref)]
pub struct MultiplayerFeed(pub Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<Multiplayer>>>);

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Deref)]
pub struct FreenetEndpoint(pub u16);

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct Multiplayer {
    pub catalogue: BTreeMap<RoomName, RoomRecord>,
    pub room: Option<Room>,
}
