use freenet_stdlib::client_api::{ContractResponse, HostResponse};

use crate::discovery;

pub async fn recv_response(
    client: &mut discovery::Client,
) -> Result<HostResponse, discovery::Error> {
    loop {
        match client.recv().await? {
            HostResponse::ContractResponse(ContractResponse::UpdateNotification { .. }) => {}
            other => return Ok(other),
        }
    }
}
// no test_usage necessary
