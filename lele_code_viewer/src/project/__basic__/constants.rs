use std::time::Duration;

pub const RECENT_WINDOW: Duration = Duration::from_mins(10);
pub const IDLE_UNWATCH: Duration = Duration::from_mins(15);
pub const SWEEP_EVERY: Duration = Duration::from_secs(60);
pub const MAX_WATCH_DIRS: usize = 2000;
pub const MAX_DIFF_BYTES: u64 = 2_000_000;
