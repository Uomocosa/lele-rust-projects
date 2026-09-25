use tracing::info;

use super::super::directory::room_catalog::RoomCatalog;
use super::super::gossip::hint_store::HintStore;
use super::super::gossip::hint_union::hint_union;
use super::super::params::remote_peer_id::RemotePeerId;
use super::super::params::room_name::RoomName;
use super::epoch_secs::epoch_secs;

pub fn publish_pre_get_union(
    expected_tx: &tokio::sync::mpsc::UnboundedSender<Vec<RemotePeerId>>,
    slots: &RoomCatalog,
    peer_id: &RemotePeerId,
    room: &RoomName,
) {
    let union = hint_union(slots, &HintStore::default(), peer_id, epoch_secs());
    info!(target: "room_lobby", room = %room.as_str(), peers = union.len(), "discovery: expected hint-union published before roster get");
    expected_tx.send(union).ok();
}

// no test_usage necessary
