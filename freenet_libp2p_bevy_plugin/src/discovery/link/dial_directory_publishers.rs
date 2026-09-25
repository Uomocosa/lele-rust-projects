use crate::p2p;

use super::super::directory::DirectoryState;
use super::dial_hint_raw::dial_hint_raw;
use super::directory_hints::directory_hints;
use super::maps::{AttemptedMap, ConnectedMap, StaggerMap};

pub fn dial_directory_publishers(
    net_tx: &tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    attempted: &mut AttemptedMap,
    connected: &ConnectedMap,
    staggers: &mut StaggerMap,
    peer_id: &str,
    mode: p2p::TransportMode,
    slots: &DirectoryState,
) {
    for hint in directory_hints(slots) {
        dial_hint_raw(net_tx, attempted, connected, staggers, peer_id, mode, &hint);
    }
}

// no test_usage necessary
