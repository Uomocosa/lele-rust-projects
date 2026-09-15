use freenet_stdlib::client_api::{ClientRequest, ContractRequest};
use freenet_stdlib::prelude::*;
use tracing::info;

use crate::discovery;

const SPLIT_AFTER_SECS: u64 = 30;
const BRIDGE_INTERVAL_SECS: u64 = 30;

/// # Errors
/// Returns `Error` if the bridge subscribe or re-put fails.
pub fn bridge_tick(
    roster: &mut discovery::Roster,
    now: std::time::Instant,
) -> Result<(), discovery::Error> {
    if !bridge_due(roster.foreign_seen, roster.last_bridge, now) {
        return Ok(());
    }
    roster.last_bridge = Some(now);
    info!(target: "clicker", own = *roster.own, "roster bridge: split suspected");
    attempt_subscribe(roster)?;
    attempt_reput(roster)?;
    Ok(())
}

// needed helper: sends one routed subscribe to plug into the other replica group
fn attempt_subscribe(roster: &discovery::Roster) -> Result<(), discovery::Error> {
    let instance_id = *roster.contract_key.id();
    let summary = StateSummary::from(bincode::serialize(&roster.slots)?);
    let sub_req = ContractRequest::Subscribe {
        key: instance_id,
        summary: Some(summary),
    };
    roster.client.send(&ClientRequest::ContractOp(sub_req))?;
    Ok(())
}

// needed helper: re-puts merged slots so a lone replica rejoins the hosting set
fn attempt_reput(roster: &discovery::Roster) -> Result<(), discovery::Error> {
    let put_req = ContractRequest::Put {
        contract: roster.contract.clone(),
        state: WrappedState::new(bincode::serialize(&roster.slots)?),
        related_contracts: RelatedContracts::default(),
        subscribe: true,
        blocking_subscribe: false,
    };
    roster.client.send(&ClientRequest::ContractOp(put_req))?;
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
