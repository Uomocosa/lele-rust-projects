use std::time::Instant;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest};
use freenet_stdlib::prelude::*;
use tracing::info;

use crate::discovery;

const SPLIT_AFTER_SECS: u64 = 30;
const BRIDGE_INTERVAL_SECS: u64 = 30;

pub fn bridge_tick(
    roster: &mut discovery::link::RosterClient,
    now: Instant,
) -> Result<(), discovery::Error> {
    if !bridge_due(roster.foreign_seen, roster.last_bridge, now) {
        return Ok(());
    }
    roster.last_bridge = Some(now);
    info!(target: "room_lobby", own = *roster.own, "roster bridge: split suspected");
    let instance_id = *roster.contract_key.id();
    let summary = StateSummary::from(bincode::serialize(&roster.slots)?);
    let sub_req = ContractRequest::Subscribe {
        key: instance_id,
        summary: Some(summary),
    };
    roster.client.send(&ClientRequest::ContractOp(sub_req))?;
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
fn bridge_due(foreign_seen: Option<Instant>, last_bridge: Option<Instant>, now: Instant) -> bool {
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

#[cfg(test)]
mod tests {
    use std::time::Instant;

    #[test]
    fn test_usage() {
        let now = Instant::now();
        assert!(super::bridge_due(None, None, now));
        assert!(!super::bridge_due(Some(now), Some(now), now));
    }
}
