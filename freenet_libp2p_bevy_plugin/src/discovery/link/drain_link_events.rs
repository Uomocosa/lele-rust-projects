use crate::p2p;

use std::time::Instant;

use tracing::warn;

use super::count_link::count_link;
use super::decrement_link::decrement_link;
use super::drain_tap::drain_tap;
use super::run_context::RunContext;

pub fn drain_link_events(ctx: &mut RunContext) {
    while let Ok(event) = ctx.tap_rx.try_recv() {
        match event {
            p2p::TapEvent::PeerConnected(peer) => {
                let fresh = !ctx.connected.contains_key(&peer);
                count_link(ctx, peer.clone());
                if fresh {
                    ctx.attempted.insert(peer, Instant::now());
                }
            }
            p2p::TapEvent::PeerDisconnected(peer) => {
                decrement_link(ctx, &peer);
            }
            p2p::TapEvent::DialFailed { peer_id, .. } => {
                ctx.staggers.remove(&peer_id);
                ctx.pex.remove(peer_id.as_str());
                warn!(target: "room_lobby", peer = %peer_id, "discovery: pruned dial-failed peer");
            }
            other => drain_tap(ctx, other),
        }
    }
}

// no test_usage necessary
