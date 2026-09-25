use std::collections::BTreeMap;

use freenet_stdlib::prelude::ContractKey;
use serde::{Deserialize, Serialize};

use super::super::config::Config;
use super::enums::DiscoveryStatus;
use super::messages::{Command, Event};
use super::newtypes::{EpochSecs, RemotePeerId, RoomName};
use super::resources::{FreenetEndpoint, Multiplayer};
use super::type_aliases::{Members, RoomCatalogue};
use crate::discovery;
use crate::p2p;

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

#[cfg(test)]
mod tests {
    use super::{Member, Presence, Room, RoomRecord};
    use crate::discovery;

    #[test]
    fn test_usage() {
        let presence = Presence {
            addrs: vec!["/ip4/127.0.0.1/tcp/4001".to_string()],
            updated_at: discovery::id::EpochSecs(7),
        };
        let bytes = bincode::serialize(&presence).unwrap_or_default();
        let decoded: Presence = bincode::deserialize(&bytes).expect("decodes");
        assert_eq!(decoded, presence);

        let mut members = std::collections::BTreeMap::new();
        members.insert(discovery::id::RemotePeerId("peer".to_string()), presence);
        let record = RoomRecord {
            capacity: 8,
            members,
        };
        assert_eq!(record.capacity, 8);

        let member = Member {
            presence: record
                .members
                .get(&discovery::id::RemotePeerId("peer".to_string()))
                .cloned()
                .expect("member"),
            status: discovery::room_peers::DiscoveryStatus::Known,
        };
        assert!(matches!(
            member.status,
            discovery::room_peers::DiscoveryStatus::Known
        ));

        let room = Room {
            name: discovery::id::RoomName("room-a".to_string()),
            members: discovery::room_peers::Members::new(),
        };
        assert!(room.members.is_empty());
    }
}
