use tracing::info;

use super::super::constants;
use super::super::membership::roster_state::RosterState;
use super::super::params::player_id::PlayerId;
use super::super::params::remote_peer_id::RemotePeerId;
use super::epoch_secs::epoch_secs;

pub fn send_expected(
    expected_tx: &tokio::sync::mpsc::UnboundedSender<Vec<RemotePeerId>>,
    slots: &RosterState,
    own: PlayerId,
) {
    let now = epoch_secs();
    let mut peers: Vec<RemotePeerId> = slots
        .iter()
        .filter(|(id, entry)| {
            **id != own
                && !entry.peer_id.is_empty()
                && now.saturating_sub(*entry.updated_at) <= constants::STALE_ENTRY_SECS
        })
        .map(|(_, entry)| entry.peer_id.clone())
        .collect();
    peers.sort();
    peers.dedup();
    info!(target: "room_lobby", peers = peers.len(), "discovery: expected set published");
    expected_tx.send(peers).ok();
}

// no test_usage necessary
