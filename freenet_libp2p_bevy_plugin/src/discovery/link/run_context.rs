use std::collections::BTreeMap;
use std::time::Instant;

use super::super::directory::room_catalog::RoomCatalog;
use super::super::directory::room_payload::RoomPayload;
use super::super::gossip::hint_store::HintStore;
use super::super::params::contract_params::ContractParams;
use super::super::params::player_id::PlayerId;
use super::super::params::remote_peer_id::RemotePeerId;
use super::super::params::room_name::RoomName;
use super::super::params::unique_game_id::UniqueGameId;
use super::catalog_client::CatalogClient;
use super::maps::{AttemptedMap, ConnectedMap, StaggerMap};
use super::roster_client::RosterClient;
use crate::p2p;

pub struct RunContext {
    pub net_tx: tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    pub roster: RosterClient,
    pub directory: CatalogClient,
    pub room: RoomName,
    pub room_params: ContractParams,
    pub peer_id: RemotePeerId,
    pub tap_rx: tokio::sync::mpsc::UnboundedReceiver<p2p::TapEvent>,
    pub observed_rx: tokio::sync::watch::Receiver<Option<Vec<String>>>,
    pub id: UniqueGameId,
    pub last_addrs: Vec<String>,
    pub last_announce: Instant,
    pub last_gossip: Instant,
    pub last_redial: Instant,
    pub last_directory: Instant,
    pub last_pex: Instant,
    pub connected: ConnectedMap,
    pub attempted: AttemptedMap,
    pub staggers: StaggerMap,
    pub transport: p2p::TransportMode,
    pub directory_tx: tokio::sync::mpsc::UnboundedSender<RoomCatalog>,
    pub expected_tx: tokio::sync::mpsc::UnboundedSender<Vec<RemotePeerId>>,
    pub pex: HintStore,
    pub known_rooms: BTreeMap<RoomName, RoomPayload>,
    pub room_requests: tokio::sync::mpsc::UnboundedReceiver<RoomName>,
    pub room_tx: tokio::sync::watch::Sender<Option<RoomName>>,
    pub ws_port: u16,
    pub own: PlayerId,
    pub pending_switch: Option<(RoomName, u32)>,
}
