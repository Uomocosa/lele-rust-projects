use freenet_stdlib::client_api::{ContractResponse, HostResponse};

pub async fn recv_response(
    client: &mut crate::FreenetClient,
) -> Result<HostResponse, crate::ClientError> {
    loop {
        match crate::FreenetClientMethod::recv(client).await? {
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
