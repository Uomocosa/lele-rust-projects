use tracing::info;

use super::super::gossip::hint_union::hint_union;
use super::super::gossip::merge_peer_hints::merge_peer_hints;
use super::dial_hint::dial_hint;
use super::directory_hints::directory_hints;
use super::epoch_secs::epoch_secs;
use super::run_context::RunContext;

pub fn send_hint_union(ctx: &mut RunContext) {
    let union = hint_union(&ctx.directory.slots, &ctx.pex, &ctx.peer_id, epoch_secs());
    info!(target: "room_lobby", peers = union.len(), "discovery: expected hint-union published");
    ctx.expected_tx.send(union).ok();
    let mut hints = directory_hints(&ctx.directory.slots);
    hints.extend(ctx.pex.values().cloned());
    for hint in merge_peer_hints(hints) {
        dial_hint(ctx, &hint);
    }
}

// no test_usage necessary
