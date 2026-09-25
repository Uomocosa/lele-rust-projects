use std::time::Instant;

use tracing::warn;

use super::dial_hint::dial_hint;
use super::directory_hints::directory_hints;
use super::run_context::RunContext;
use super::send_merged_directory::send_merged_directory;

pub async fn refresh_directory(ctx: &mut RunContext) {
    match ctx.directory.poll().await {
        Ok(slots) => {
            ctx.directory.slots = slots;
            send_merged_directory(ctx);
            for hint in directory_hints(&ctx.directory.slots) {
                ctx.pex.insert(hint.clone());
                dial_hint(ctx, &hint);
            }
        }
        Err(e) => {
            warn!(target: "room_lobby", error = %e, "discovery: directory poll failed");
        }
    }
    if ctx
        .directory
        .publish_room(&ctx.room, &ctx.room_params, &ctx.peer_id, &ctx.last_addrs)
        .is_err()
    {
        warn!(target: "room_lobby", "discovery: directory refresh failed");
    }
    if ctx.directory.bridge_tick(Instant::now()).is_err() {
        warn!(target: "room_lobby", "discovery: directory bridge failed");
    }
}

// no test_usage necessary
