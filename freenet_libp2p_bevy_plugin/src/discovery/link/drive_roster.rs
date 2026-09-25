use std::time::{Duration, Instant};

use tracing::{info, warn};

use super::super::constants;
use super::super::dial::should_switch::should_switch;
use super::dial_entry::dial_entry;
use super::drain_link_events::drain_link_events;
use super::epoch_secs::epoch_secs;
use super::fire_due_staggers::fire_due_staggers;
use super::pex_msg::PexMsg;
use super::publish_pex::publish_pex;
use super::publish_slots::publish_slots;
use super::redial_missing::redial_missing;
use super::refresh_directory::refresh_directory;
use super::refresh_observed::refresh_observed;
use super::run_context::RunContext;
use super::switch_room::switch_room;

pub async fn drive_roster(ctx: &mut RunContext) {
    loop {
        while let Ok(room) = ctx.room_requests.try_recv() {
            if should_switch(&ctx.room, &room) {
                ctx.pending_switch = Some((room, 0));
            }
        }
        if let Some((room, attempts)) = ctx.pending_switch.take() {
            if attempts == 0 || attempts % 10 == 0 {
                info!(target: "room_lobby", room = %room, attempts, "discovery: switching room");
            }
            if !switch_room(ctx, &room).await {
                ctx.pending_switch = Some((room, attempts.saturating_add(1)));
            }
        }
        drain_link_events(ctx);
        match ctx.roster.poll().await {
            Ok(fresh) => {
                for entry in fresh {
                    dial_entry(ctx, &entry);
                }
            }
            Err(e) => {
                warn!(target: "room_lobby", error = %e, "discovery: poll failed");
            }
        }
        if ctx.last_directory.elapsed().as_secs() >= constants::DIRECTORY_TICK_SECS {
            ctx.last_directory = Instant::now();
            refresh_directory(ctx).await;
        }
        if ctx.last_gossip.elapsed().as_secs_f64() >= constants::ROSTER_HEARTBEAT_SECS {
            ctx.last_gossip = Instant::now();
            publish_slots(ctx);
        }
        if ctx.last_redial.elapsed().as_secs() >= constants::REDIAL_SECS {
            ctx.last_redial = Instant::now();
            redial_missing(ctx);
        }
        if ctx.last_pex.elapsed().as_secs() >= constants::PEX_INTERVAL_SECS {
            ctx.last_pex = Instant::now();
            ctx.pex.prune(epoch_secs());
            publish_pex(&ctx.net_tx, &ctx.namespace, &PexMsg::Ask);
        }
        fire_due_staggers(ctx);
        if ctx.observed_rx.has_changed().unwrap_or(false) {
            refresh_observed(ctx);
        }
        if ctx.roster.bridge_tick(Instant::now()).is_err() {
            warn!(target: "room_lobby", "discovery: bridge failed");
        }
        if ctx.roster.announce().is_err() {
            warn!(target: "room_lobby", "discovery: announce failed");
        }
        tokio::time::sleep(Duration::from_secs(constants::TICK_SECS)).await;
    }
}

// no test_usage necessary
