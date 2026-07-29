use std::time::Duration;

use freenet_stdlib::client_api::HostResponse;

use crate::methods::freenet_client as fc_method;

pub async fn recv_timeout(
    client: &mut crate::structs::freenet_client::FreenetClient,
    timeout: Duration,
) -> Option<Result<HostResponse, crate::ClientError>> {
    tokio::time::timeout(timeout, fc_method::recv(client))
        .await
        .ok()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {}
}
