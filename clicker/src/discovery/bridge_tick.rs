use freenet_stdlib::client_api::{ClientRequest, ContractRequest};
use freenet_stdlib::prelude::*;
use tracing::info;

use crate::discovery;

const SPLIT_AFTER_SECS: u64 = 30;
const BRIDGE_INTERVAL_SECS: u64 = 30;

/// # Errors
/// Returns `Error` if the bridge subscribe fails.
pub async fn bridge_tick(
    roster: &mut discovery::Roster,
    now: std::time::Instant,
) -> Result<(), discovery::Error> {
    if !bridge_due(roster.foreign_seen, roster.last_bridge, now) {
        return Ok(());
    }
    roster.last_bridge = Some(now);
    info!(target: "clicker", own = *roster.own, "roster bridge: split suspected");
    let instance_id = *roster.contract_key.id();
    let summary = StateSummary::from(bincode::serialize(&roster.slots)?);
    let sub_req = ContractRequest::Subscribe {
        key: instance_id,
        summary: Some(summary),
    };
    roster.client.send(&ClientRequest::ContractOp(sub_req))?;
    Ok(())
}

// needed helper: decides whether a routed re-subscribe is due
fn bridge_due(
    foreign_seen: Option<std::time::Instant>,
    last_bridge: Option<std::time::Instant>,
    now: std::time::Instant,
) -> bool {
    let silent = foreign_seen.is_none_or(|t| {
        now.checked_duration_since(t)
            .is_none_or(|d| d.as_secs() >= SPLIT_AFTER_SECS)
    });
    let due = last_bridge.is_none_or(|t| {
        now.checked_duration_since(t)
            .is_none_or(|d| d.as_secs() >= BRIDGE_INTERVAL_SECS)
    });
    silent && due
}
// no test_usage necessary
