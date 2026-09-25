use std::time::Instant;

use tracing::{info, warn};

use super::super::params::resolve_params::resolve_params;
use super::roster_client::RosterClient;
use super::run_context::RunContext;
use super::send_expected::send_expected;
use super::send_hint_union::send_hint_union;
use super::subscribe_topics::subscribe_topics;

pub async fn switch_room(ctx: &mut RunContext, room: &str) -> bool {
    send_hint_union(ctx);
    let params = resolve_params(&ctx.namespace, room, ctx.params_override.as_deref());
    let attempt = Instant::now();
    match RosterClient::connect(
        "127.0.0.1",
        ctx.ws_port,
        super::contract_wasm::contract_wasm(),
        &params,
        ctx.own,
        &ctx.peer_id,
        &ctx.last_addrs,
    )
    .await
    {
        Ok(roster) => {
            info!(target: "room_lobby", room = %room, slots = roster.slots.len(), elapsed_ms = attempt.elapsed().as_millis(), "discovery: roster fetched");
            ctx.roster = roster;
            send_expected(&ctx.expected_tx, &ctx.roster.slots, ctx.own);
            ctx.room = room.to_string();
            ctx.room_params = params;
            ctx.room_tx.send_replace(Some(room.to_string()));
            ctx.attempted.clear();
            ctx.staggers.clear();
            let now = Instant::now();
            ctx.last_announce = now;
            ctx.last_gossip = now;
            ctx.last_redial = now;
            subscribe_topics(&ctx.net_tx, &ctx.namespace, room);
            if ctx
                .directory
                .publish_room(room, &ctx.room_params, &ctx.peer_id, &ctx.last_addrs)
                .is_err()
            {
                warn!(target: "room_lobby", "discovery: room publish failed on switch");
            }
            if ctx.roster.announce().is_err() {
                warn!(target: "room_lobby", "discovery: announce failed on switch");
            }
            info!(target: "room_lobby", room = %room, "discovery: switched room");
            true
        }
        Err(e) => {
            warn!(target: "room_lobby", error = %e, "discovery: roster switch failed");
            false
        }
    }
}

// no test_usage necessary
