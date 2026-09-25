use super::super::gossip::peer_hint::PeerHint;
use super::super::membership::peer_entry::PeerEntry;
use super::dial_entry::dial_entry;
use super::dial_hint::dial_hint;
use super::run_context::RunContext;

pub fn redial_missing(ctx: &mut RunContext) {
    let entries: Vec<PeerEntry> = ctx
        .roster
        .slots
        .iter()
        .filter(|(id, _)| **id != ctx.own)
        .map(|(_, entry)| entry.clone())
        .collect();
    for entry in &entries {
        if ctx.connected.contains_key(entry.peer_id.as_str()) {
            continue;
        }
        dial_entry(ctx, entry);
    }
    let hints: Vec<PeerHint> = ctx.pex.values().cloned().collect();
    for hint in &hints {
        if ctx.connected.contains_key(hint.peer_id.as_str()) {
            continue;
        }
        dial_hint(ctx, hint);
    }
}

// no test_usage necessary
