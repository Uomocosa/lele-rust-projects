use std::time::Duration;

use freenet_stdlib::client_api::{ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use crate::FreenetClient;

pub async fn recv_notification(client: &mut FreenetClient, timeout: Duration) -> Option<u64> {
    match tokio::time::timeout(timeout, client.recv()).await {
        Ok(Ok(HostResponse::ContractResponse(ContractResponse::UpdateNotification {
            update,
            ..
        }))) => {
            let count = match &update {
                UpdateData::State(s) => bincode::deserialize(s.as_ref()).unwrap_or(0),
                UpdateData::Delta(d) => bincode::deserialize(d.as_ref()).unwrap_or(0),
                _ => 0,
            };
            Some(count)
        }
        _ => None,
    }
}
