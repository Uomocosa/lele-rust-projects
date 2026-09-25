use std::time::Instant;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest};
use freenet_stdlib::prelude::*;

use crate::discovery;

const BRIDGE_INTERVAL_SECS: u64 = 30;

pub fn bridge_tick(
    directory: &mut discovery::link::CatalogClient,
    now: Instant,
) -> Result<(), discovery::Error> {
    let due = directory.last_bridge.is_none_or(|t| {
        now.checked_duration_since(t)
            .is_none_or(|d| d.as_secs() >= BRIDGE_INTERVAL_SECS)
    });
    if !due {
        return Ok(());
    }
    directory.last_bridge = Some(now);
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        std::hint::black_box(super::bridge_tick);
    }
}
