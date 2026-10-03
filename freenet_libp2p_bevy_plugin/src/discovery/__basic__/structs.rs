use std::collections::BTreeMap;

use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::discovery;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct Presence {
    pub addrs: Vec<String>,
    pub updated_at: discovery::EpochSecs,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct RoomRecord {
    pub capacity: u16,
    pub members: BTreeMap<discovery::PeerId, Presence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub struct Member {
    pub presence: Presence,
    pub status: discovery::LinkStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub struct Room {
    pub name: discovery::RoomName,
    pub members: discovery::Members,
}

pub struct Channels {
    pub net: discovery::libp2p::Link,
    pub commands: UnboundedReceiver<discovery::Command>,
    pub snapshots: UnboundedSender<discovery::Snapshot>,
    pub events: UnboundedSender<discovery::Event>,
}
