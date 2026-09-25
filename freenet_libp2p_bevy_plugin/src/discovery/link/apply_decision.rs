use std::time::Instant;

use super::super::dial::decision::Decision;
use super::super::membership::peer_entry::PeerEntry;
use super::dial_preferred::dial_preferred;
use super::force_entry::force_entry;
use super::run_context::RunContext;

pub fn apply_decision(ctx: &mut RunContext, entry: &PeerEntry, decision: Decision) {
    match decision {
        Decision::Wait => {
            ctx.attempted
                .entry(entry.peer_id.clone())
                .or_insert_with(Instant::now);
        }
        Decision::Dial => {
            ctx.attempted.insert(entry.peer_id.clone(), Instant::now());
            dial_preferred(ctx, entry);
        }
        Decision::ForceDial => {
            ctx.attempted.insert(entry.peer_id.clone(), Instant::now());
            force_entry(ctx, entry);
        }
    }
}

// no test_usage necessary
