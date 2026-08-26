use crate::clicker_error;
use std::collections::BTreeSet;
use std::time::Duration;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use crate::set_client;
use clicker_error::ClickerError as Ce;

// needed helper:
fn absorb_set(set: &mut BTreeSet<u64>, bytes: &[u8]) {
    if let Ok(incoming) = bincode::deserialize::<BTreeSet<u64>>(bytes) {
        *set = incoming;
    }
}

// needed helper:
fn mine_value(tag: u64, seq: u64) -> u64 {
    (tag << 32) | seq
}

pub async fn tick(client: &mut set_client::SetClient) -> Result<u64, Ce> {
    while let Some(result) = client.client.recv_timeout(Duration::from_millis(10)).await {
        if let Ok(HostResponse::ContractResponse(ContractResponse::UpdateNotification {
            update,
            ..
        })) = result
        {
            match update {
                UpdateData::State(s) => absorb_set(&mut client.set, s.as_ref()),
                UpdateData::Delta(d) => absorb_set(&mut client.set, d.as_ref()),
                _ => {}
            }
        }
    }

    client.seq = client.seq.wrapping_add(1);
    client.set.insert(mine_value(client.tag, client.seq));
    let new_state = State::from(bincode::serialize(&client.set)?);
    let update_req = ContractRequest::Update {
        key: client.contract_key,
        data: UpdateData::State(new_state),
    };
    client
        .client
        .send(ClientRequest::ContractOp(update_req))
        .await?;

    match client.client.recv_response().await? {
        HostResponse::ContractResponse(ContractResponse::UpdateResponse { .. }) => {}
        other => return Err(Ce::UnexpectedResponse(format!("{other:?}"))),
    }

    Ok(client.set.len() as u64)
}

#[cfg(test)]
mod tests {
    use super::{absorb_set, mine_value};

    #[test]
    fn test_usage() {
        let mut set = std::collections::BTreeSet::new();
        let bytes = bincode::serialize(&std::collections::BTreeSet::from([1u64, 2])).unwrap();
        absorb_set(&mut set, &bytes);
        assert_eq!(set.len(), 2);
        assert_eq!(mine_value(3, 7), (3u64 << 32) | 7);
    }
}
