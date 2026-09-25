use crate::p2p;

use super::super::gossip::peer_hint::PeerHint;
use super::absorb_pex::absorb_pex;
use super::absorb_roster_gossip::absorb_roster_gossip;
use super::dial_hint::dial_hint;
use super::epoch_secs::epoch_secs;
use super::is_roster_topic::is_roster_topic;
use super::pex_topic::pex_topic;
use super::run_context::RunContext;

pub fn drain_tap(ctx: &mut RunContext, event: p2p::TapEvent) {
    match event {
        p2p::TapEvent::Gossip { topic, from, data } => {
            if topic == pex_topic(&ctx.namespace) {
                absorb_pex(ctx, &from, &data);
            } else if is_roster_topic(&ctx.namespace, &topic) {
                absorb_roster_gossip(ctx, &from, &data);
            }
        }
        p2p::TapEvent::LobbyProviders { peers, .. } => {
            for peer in peers {
                dial_hint(
                    ctx,
                    &PeerHint {
                        peer_id: peer,
                        addrs: Vec::new(),
                        rooms: Vec::new(),
                        updated_at: epoch_secs(),
                    },
                );
            }
        }
        _ => {}
    }
}

// no test_usage necessary
