use freenet_stdlib::client_api::{ClientRequest, ContractRequest};
use freenet_stdlib::prelude::*;

use crate::discovery;

const BRIDGE_INTERVAL_SECS: u64 = 30;

/// # Errors
/// Returns `Error` if the directory bridge subscribe or re-put fails.
pub fn directory_bridge_tick(
    directory: &discovery::Directory,
    last_bridge: &mut Option<std::time::Instant>,
    now: std::time::Instant,
) -> Result<(), discovery::Error> {
    let due = last_bridge.is_none_or(|t| {
        now.checked_duration_since(t)
            .is_none_or(|d| d.as_secs() >= BRIDGE_INTERVAL_SECS)
    });
    if !due {
        return Ok(());
    }
    *last_bridge = Some(now);
    let instance_id = *directory.contract_key.id();
    let summary = StateSummary::from(bincode::serialize(&directory.slots)?);
    let sub_req = ContractRequest::Subscribe {
        key: instance_id,
        summary: Some(summary),
    };
    directory.client.send(&ClientRequest::ContractOp(sub_req))?;
    let put_req = ContractRequest::Put {
        contract: directory.contract.clone(),
        state: WrappedState::new(bincode::serialize(&directory.slots)?),
        related_contracts: RelatedContracts::default(),
        subscribe: true,
        blocking_subscribe: false,
    };
    directory.client.send(&ClientRequest::ContractOp(put_req))?;
    Ok(())
}
// no test_usage necessary
