use freenet_stdlib::client_api::{ContractResponse, HostResponse};

use super::freenet_client_recv;

pub async fn recv_response(
    client: &mut crate::freenet_client::FreenetClient,
) -> Result<HostResponse, crate::ClientError> {
    loop {
        match freenet_client_recv::recv(client).await? {
            HostResponse::ContractResponse(ContractResponse::UpdateNotification { .. }) => {
                continue;
            }
            other => return Ok(other),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {}
}
