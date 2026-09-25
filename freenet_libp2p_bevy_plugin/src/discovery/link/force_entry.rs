use crate::p2p;

use tracing::info;

use super::super::constants;
use super::super::membership::peer_entry::PeerEntry;
use super::epoch_secs::epoch_secs;
use super::run_context::RunContext;
use super::with_loopback::with_loopback;

pub fn force_entry(ctx: &RunContext, entry: &PeerEntry) {
    if entry.addrs.is_empty() {
        return;
    }
    if epoch_secs().saturating_sub(entry.updated_at) > constants::STALE_ENTRY_SECS {
        return;
    }
    info!(target: "room_lobby", peer = %entry.peer_id, "discovery: force-dialing peer");
    ctx.net_tx
        .send(p2p::NetCommand::DialForce {
            peer_id: entry.peer_id.clone(),
            addrs: with_loopback(&entry.addrs),
        })
        .ok();
}

// no test_usage necessary
