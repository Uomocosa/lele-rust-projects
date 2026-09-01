use crate::client_error;
use crate::freenet_client;
use tokio_tungstenite::tungstenite::Message;

use freenet_stdlib::client_api::ClientRequest;

use client_error::ClientError as Ce;

/// # Errors
/// Returns `ClientError` if serialization fails or the channel is closed.
pub async fn send(
    client: &freenet_client::FreenetClient,
    request: ClientRequest<'_>,
) -> Result<(), Ce> {
    let bytes = bincode::serialize(&request)?;
    client
        .write
        .send(Message::Binary(bytes.into()))
        .map_err(|_| Ce::ChannelSend)?;
    Ok(())
}

#[cfg(all(test, feature = "dev"))]
mod tests {
    use crate::testing::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let node = TestNode::start().await.unwrap();
        let mut client = connect(node.port).await.unwrap();
        let wasm = load_wasm();
        let _key = deploy(&mut client, &wasm).await.unwrap();
    }
}
