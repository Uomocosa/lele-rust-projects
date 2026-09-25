use super::run_context::RunContext;

pub fn count_link(ctx: &mut RunContext, peer: String) {
    ctx.connected
        .entry(peer)
        .and_modify(|count| *count = count.saturating_add(1))
        .or_insert(1);
}

// no test_usage necessary
