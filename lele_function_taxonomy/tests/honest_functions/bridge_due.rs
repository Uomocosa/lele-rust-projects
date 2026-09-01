use std::time::Duration;
use std::time::Instant;

pub fn bridge_due(
    foreign_seen: Option<Instant>,
    last_bridge: Option<Instant>,
    now: Instant,
) -> bool {
    let silent = foreign_seen.is_none_or(|t| {
        now.checked_duration_since(t)
            .is_none_or(|d| d.as_secs() >= 30)
    });
    let due = last_bridge.is_none_or(|t| {
        now.checked_duration_since(t)
            .is_none_or(|d| d.as_secs() >= 30)
    });
    silent && due
}
