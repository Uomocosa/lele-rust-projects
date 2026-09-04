use std::time::Duration;

use freenet_stdlib::client_api::HostResponse;

use crate::freenet_client;
use crate::freenet_client_recv;

pub async fn recv_with_timeout(
    client: &mut freenet_client::FreenetClient,
    dur: Duration,
) -> Option<Result<HostResponse, String>> {
    let fut = freenet_client_recv::recv(client);
    tokio::time::timeout(dur, fut).await.ok()
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_usage() {
        let _ = stringify!(recv_with_timeout);
    }
}
