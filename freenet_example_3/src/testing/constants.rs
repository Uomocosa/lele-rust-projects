use std::time::Duration;

pub const NOTIFY_TIMEOUT: Duration = Duration::from_secs(10);
pub const WAIT_TIMEOUT: Duration = Duration::from_secs(10);
pub const GATEWAY_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
pub const DEPLOY_TIMEOUT: Duration = Duration::from_secs(15);
pub const TICK_INTERVAL: Duration = Duration::from_secs(1);
pub const CONNECT_POLL_INTERVAL: Duration = Duration::from_millis(200);
pub const DRAIN_POLL_INTERVAL: Duration = Duration::from_millis(50);
pub const MERGE_TICKS_MIN: u64 = 30;

// no test_usage necessary
