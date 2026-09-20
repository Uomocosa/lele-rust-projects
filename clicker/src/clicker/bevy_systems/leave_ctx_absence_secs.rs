use crate::clicker;

pub fn absence_secs(ctx: &mut clicker::bevy_systems::LeaveCtx<'_, '_>, owner: u64) -> f64 {
    let now = ctx.time.elapsed().as_secs_f64();
    let since = *ctx.absent.entry(owner).or_insert(now);
    now - since
}

// no test_usage necessary
