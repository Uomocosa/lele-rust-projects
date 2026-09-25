use tracing::info;

use super::super::directory::DirectoryState;
use super::super::gossip::hint_store::HintStore;
use super::super::gossip::hint_union::hint_union;
use super::epoch_secs::epoch_secs;

pub fn publish_pre_get_union(
    expected_tx: &tokio::sync::mpsc::UnboundedSender<Vec<String>>,
    slots: &DirectoryState,
    peer_id: &str,
    room: &str,
) {
    let union = hint_union(slots, &HintStore::default(), peer_id, epoch_secs());
    info!(target: "room_lobby", room = %room, peers = union.len(), "discovery: expected hint-union published before roster get");
    expected_tx.send(union).ok();
}

// no test_usage necessary
