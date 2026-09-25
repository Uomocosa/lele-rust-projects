use std::collections::VecDeque;
use std::time::{Duration, Instant};

use super::super::constants;
use super::super::dial::rank_addrs::rank_addrs;
use super::super::membership::peer_entry::PeerEntry;
use super::dial_addrs::dial_addrs;
use super::run_context::RunContext;
use super::with_loopback::with_loopback;

pub fn dial_preferred(ctx: &mut RunContext, entry: &PeerEntry) {
    if ctx.connected.contains_key(entry.peer_id.as_str()) {
        ctx.staggers.remove(entry.peer_id.as_str());
        return;
    }
    let ranked = rank_addrs(&with_loopback(&entry.addrs), ctx.transport);
    let mut queue: VecDeque<String> = ranked.into();
    let Some(first) = queue.pop_front() else {
        return;
    };
    dial_addrs(&ctx.net_tx, ctx.peer_id.as_str(), &[first], &entry.addrs);
    if queue.is_empty() {
        ctx.staggers.remove(entry.peer_id.as_str());
        return;
    }
    let due = Instant::now()
        .checked_add(Duration::from_secs(constants::STAGGER_SECS))
        .unwrap_or_else(Instant::now);
    ctx.staggers.insert((*entry.peer_id).clone(), (queue, due));
}

// no test_usage necessary
