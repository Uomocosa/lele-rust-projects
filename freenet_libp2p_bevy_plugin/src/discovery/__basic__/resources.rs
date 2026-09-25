use std::collections::BTreeMap;
use std::sync::Mutex;

use bevy::prelude::Resource;
use derive_more::Deref;

use super::messages::{Command, Event};
use super::newtypes::RoomName;
use super::structs::{Room, RoomRecord};

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

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::{CommandSender, EventFeed, FreenetEndpoint, Multiplayer, MultiplayerFeed};
    use crate::discovery;

    #[test]
    fn test_usage() {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let sender = CommandSender(tx);
        assert!(sender.send(discovery::Command::Leave).is_ok());

        let (_tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let feed = EventFeed(Mutex::new(Some(rx)));
        assert!(feed.lock().is_ok_and(|guard| guard.is_some()));

        let (_tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let feed = MultiplayerFeed(Mutex::new(Some(rx)));
        assert!(feed.lock().is_ok_and(|guard| guard.is_some()));

        assert_eq!(*FreenetEndpoint(7509), 7509);

        let multiplayer = Multiplayer::default();
        assert!(multiplayer.catalogue.is_empty());
        assert!(multiplayer.room.is_none());
    }
}
