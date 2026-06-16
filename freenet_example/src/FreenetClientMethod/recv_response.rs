use freenet_stdlib::client_api::{ContractResponse, HostResponse};

use crate::ClientError;

pub async fn recv_response(client: &mut crate::FreenetClient) -> Result<HostResponse, ClientError> {
    loop {
        match client.recv().await? {
            HostResponse::ContractResponse(ContractResponse::UpdateNotification { .. }) => continue,
            other => return Ok(other),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {}
}
