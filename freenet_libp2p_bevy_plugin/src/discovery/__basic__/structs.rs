use std::collections::BTreeMap;

use freenet_stdlib::prelude::ContractKey;
use serde::{Deserialize, Serialize};

use crate::discovery;
use crate::p2p;
use discovery::basic::enums::DiscoveryStatus;
use discovery::basic::messages::{Command, Event};
use discovery::basic::newtypes::{EpochSecs, RemotePeerId, RoomName};
use discovery::basic::resources::{FreenetEndpoint, Multiplayer};
use discovery::basic::type_aliases::{Members, RoomCatalogue};
use discovery::config::Config;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Presence {
    pub addrs: Vec<String>,
    pub updated_at: EpochSecs,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomRecord {
    pub capacity: u16,
    pub members: BTreeMap<RemotePeerId, Presence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub presence: Presence,
    pub status: DiscoveryStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Room {
    pub name: RoomName,
    pub members: Members,
}

pub struct RunConfig {
    pub config: Config,
    pub endpoint: FreenetEndpoint,
    pub link: NetLink,
    pub tap: tokio::sync::mpsc::UnboundedReceiver<p2p::TapEvent>,
    pub ready: tokio::sync::watch::Receiver<Option<p2p::Ready>>,
    pub commands: tokio::sync::mpsc::UnboundedReceiver<Command>,
    pub multiplayer: tokio::sync::mpsc::UnboundedSender<Multiplayer>,
    pub events: tokio::sync::mpsc::UnboundedSender<Event>,
}

pub struct IndexClient {
    pub(crate) client: discovery::link::Client,
    pub contract_key: ContractKey,
    pub(crate) slots: RoomCatalogue,
}

pub struct NetLink {
    pub tx: tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    pub observed: tokio::sync::watch::Receiver<Option<Vec<String>>>,
}
