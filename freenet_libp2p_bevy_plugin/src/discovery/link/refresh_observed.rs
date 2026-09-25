use std::time::Instant;

use tracing::{info, warn};

use super::super::dial::observed_addrs::observed_addrs;
use super::dialable::dialable;
use super::run_context::RunContext;

pub fn refresh_observed(ctx: &mut RunContext) {
    let seen = ctx
        .observed_rx
        .borrow_and_update()
        .clone()
        .unwrap_or_default();
    let filtered = observed_addrs(seen.clone(), &ctx.last_addrs);
    if filtered != seen {
        info!(target: "room_lobby", seen = ?seen, kept = ?filtered, "discovery: ignored non-listen observed addr");
    }
    let dialable_seen = dialable(filtered);
    if dialable_seen.is_empty() || dialable_seen == ctx.last_addrs {
        return;
    }
    ctx.last_addrs.clone_from(&dialable_seen);
    ctx.roster.addrs = dialable_seen;
    ctx.last_announce = Instant::now();
    if ctx.roster.announce().is_err() {
        warn!(target: "room_lobby", "discovery: re-announce failed");
    }
}

// no test_usage necessary
