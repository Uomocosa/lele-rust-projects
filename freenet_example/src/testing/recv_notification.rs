use std::collections::BTreeMap;
use std::time::Duration;

use freenet_stdlib::client_api::{ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use crate::FreenetClient;
use crate::global_counter_client;

pub async fn recv_notification(client: &mut FreenetClient, timeout: Duration) -> Option<u64> {
    match tokio::time::timeout(timeout, client.recv()).await {
        Ok(Ok(HostResponse::ContractResponse(ContractResponse::UpdateNotification {
            update,
            ..
        }))) => {
            let count = match &update {
                UpdateData::State(s) => {
                    bincode::deserialize::<BTreeMap<global_counter_client::Pubkey, u64>>(s.as_ref())
                        .map_or(0, |m| m.values().sum())
                }
                UpdateData::Delta(d) => {
                    bincode::deserialize::<BTreeMap<global_counter_client::Pubkey, u64>>(d.as_ref())
                        .map_or(0, |m| m.values().sum())
                }
                _ => 0,
            };
            Some(count)
        }
        _ => None,
    }
}

// no test_usage necessary — exercised via integration tests
