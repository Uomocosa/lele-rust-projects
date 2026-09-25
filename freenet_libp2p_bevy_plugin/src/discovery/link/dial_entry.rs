use std::time::Instant;

use super::super::constants;
use super::super::dial::decide_dial::decide_dial;
use super::super::membership::peer_entry::PeerEntry;
use super::apply_decision::apply_decision;
use super::epoch_secs::epoch_secs;
use super::run_context::RunContext;

pub fn dial_entry(ctx: &mut RunContext, entry: &PeerEntry) {
    if entry.addrs.is_empty() {
        return;
    }
    if epoch_secs().saturating_sub(entry.updated_at) > constants::STALE_ENTRY_SECS {
        return;
    }
    let age = ctx.attempted.get(&entry.peer_id).and_then(|seen| {
        Instant::now()
            .checked_duration_since(*seen)
            .map(|d| d.as_secs())
    });
    if age.is_some_and(|seen_secs| seen_secs < constants::REDIAL_SECS) {
        return;
    }
    let decision = decide_dial(&ctx.peer_id, &entry.peer_id, age);
    apply_decision(ctx, entry, decision);
}

// no test_usage necessary
