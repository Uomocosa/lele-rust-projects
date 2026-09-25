use crate::p2p;

use std::time::Instant;

use tracing::info;

use super::super::dial::stagger_due::stagger_due;
use super::run_context::RunContext;

pub fn fire_due_staggers(ctx: &mut RunContext) {
    let due = stagger_due(Instant::now(), &ctx.connected, &mut ctx.staggers);
    for (peer, addr) in due {
        info!(target: "room_lobby", peer = %peer, addr = %addr, "discovery: staggered dialing peer");
        ctx.net_tx
            .send(p2p::NetCommand::DialForce {
                peer_id: peer,
                addrs: vec![addr],
            })
            .ok();
    }
}

// no test_usage necessary
