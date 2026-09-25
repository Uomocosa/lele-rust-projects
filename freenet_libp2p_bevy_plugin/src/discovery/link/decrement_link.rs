use super::run_context::RunContext;

pub fn decrement_link(ctx: &mut RunContext, peer: &str) {
    let drained = ctx.connected.get_mut(peer).is_some_and(|count| {
        *count = count.saturating_sub(1);
        *count == 0
    });
    if drained {
        ctx.connected.remove(peer);
    }
}

// no test_usage necessary
