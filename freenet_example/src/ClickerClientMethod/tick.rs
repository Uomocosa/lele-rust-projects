use std::time::Duration;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use crate::ClickerError as Ce;

pub async fn tick(client: &mut crate::ClickerClient) -> Result<u64, crate::ClickerError> {
    while let Some(result) = client.client.recv_timeout(Duration::from_millis(10)).await {
        if let HostResponse::ContractResponse(ContractResponse::UpdateNotification {
            update, ..
        }) = result?
        {
            client.count = match &update {
                UpdateData::State(s) => bincode::deserialize(s.as_ref()).unwrap_or(0),
                UpdateData::Delta(d) => bincode::deserialize(d.as_ref()).unwrap_or(0),
                _ => 0,
            };
        }
    }

    client.count = client.count.wrapping_add(1);
    let new_state = State::from(bincode::serialize(&client.count)?);
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

    Ok(client.count)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {}
}
