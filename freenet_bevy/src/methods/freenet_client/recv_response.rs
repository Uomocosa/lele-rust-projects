use freenet_stdlib::client_api::{ContractResponse, HostResponse};

use crate::methods::freenet_client as fc_method;

pub async fn recv_response(
    client: &mut crate::structs::freenet_client::FreenetClient,
) -> Result<HostResponse, crate::ClientError> {
    loop {
        match fc_method::recv(client).await? {
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
