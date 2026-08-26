use crate::client_error;
use crate::freenet_client;
use freenet_stdlib::client_api::HostResponse;

use client_error::ClientError as Ce;

pub async fn recv(client: &mut freenet_client::FreenetClient) -> Result<HostResponse, Ce> {
    match client.read.recv().await {
        Some(Ok(response)) => Ok(response),
        Some(Err(e)) => Err(Ce::FreenetClient(e)),
        None => Err(Ce::Disconnected),
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let node = TestNode::start().await.unwrap();
        let mut client = connect(node.port()).await.unwrap();
        let wasm = load_wasm();
        let key = deploy(&mut client, &wasm).await.unwrap();
        let count = get_count(&mut client, key).await.unwrap();
        assert_eq!(count, 0);
    }
}
