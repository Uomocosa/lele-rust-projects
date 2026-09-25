use std::collections::BTreeMap;
use std::time::Instant;

use super::super::directory::DirectoryState;
use super::super::directory::Entry;
use super::super::gossip::hint_store::HintStore;
use super::super::params::player_id::PlayerId;
use super::directory_client::DirectoryClient;
use super::maps::{AttemptedMap, ConnectedMap, StaggerMap};
use super::roster_client::RosterClient;
use crate::p2p;

pub struct RunContext {
    pub net_tx: tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    pub roster: RosterClient,
    pub directory: DirectoryClient,
    pub room: String,
    pub room_params: Vec<u8>,
    pub peer_id: String,
    pub tap_rx: tokio::sync::mpsc::UnboundedReceiver<p2p::TapEvent>,
    pub observed_rx: tokio::sync::watch::Receiver<Option<Vec<String>>>,
    pub namespace: String,
    pub params_override: Option<String>,
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
    pub directory_tx: tokio::sync::mpsc::UnboundedSender<DirectoryState>,
    pub expected_tx: tokio::sync::mpsc::UnboundedSender<Vec<String>>,
    pub pex: HintStore,
    pub known_rooms: BTreeMap<String, Entry>,
    pub room_requests: tokio::sync::mpsc::UnboundedReceiver<String>,
    pub room_tx: tokio::sync::watch::Sender<Option<String>>,
    pub ws_port: u16,
    pub own: PlayerId,
    pub pending_switch: Option<(String, u32)>,
    pub node_guard: Option<tempfile::TempDir>,
}
