use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use freenet_stdlib::client_api::{ClientRequest, ContractRequest};

use crate::discovery;

/// # Panics
/// May panic if serialization fails.
pub async fn bridge_tick(d: &mut discovery::Discovery, now: Instant) {
    if d.last_bridge
        .is_none_or(|t| now.duration_since(t) >= Duration::from_secs(30))
    {
        if d.chain.len() == d.foreign_len {
            let summary_data = discovery::StateData {
                peers: d.peers.clone(),
                chain: d.chain.clone(),
                sigs: BTreeMap::new(),
            };
            let serialized = bincode::serialize(&summary_data).unwrap_or_default();
            let summary = freenet_stdlib::prelude::StateSummary::from(serialized);
            let req = ClientRequest::ContractOp(ContractRequest::Subscribe {
                key: *d.key.id(),
                summary: Some(summary),
            });
            let _ = d.client.send(req).await;
            d.last_bridge = Some(now);
        }
        d.foreign_len = d.chain.len();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        let _ = stringify!(bridge_tick);
    }
}
