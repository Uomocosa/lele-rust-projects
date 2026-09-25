use crate::p2p;

use super::roster_topic::roster_topic;
use super::run_context::RunContext;

pub fn publish_slots(ctx: &RunContext) {
    let data = bincode::serialize(&ctx.roster.slots).unwrap_or_default();
    ctx.net_tx
        .send(p2p::NetCommand::Publish {
            topic: roster_topic(&ctx.id, &ctx.room),
            data,
        })
        .ok();
}

// no test_usage necessary
