use super::super::constants;
use super::super::gossip::peer_hint::PeerHint;
use super::super::membership::peer_entry::PeerEntry;
use super::dial_entry::dial_entry;
use super::epoch_secs::epoch_secs;
use super::run_context::RunContext;

pub fn dial_hint(ctx: &mut RunContext, hint: &PeerHint) {
    if hint.peer_id.is_empty() || hint.peer_id == ctx.peer_id || hint.addrs.is_empty() {
        return;
    }
    if epoch_secs().saturating_sub(*hint.updated_at) > constants::STALE_ENTRY_SECS {
        return;
    }
    let entry = PeerEntry {
        peer_id: hint.peer_id.clone(),
        addrs: hint.addrs.clone(),
        updated_at: hint.updated_at,
    };
    dial_entry(ctx, &entry);
}

// no test_usage necessary
