use tokio_tungstenite::tungstenite::Message;

use freenet_stdlib::client_api::ClientRequest;

use crate::ClientError as Ce;

pub async fn send(client: &crate::FreenetClient, request: ClientRequest<'_>) -> Result<(), Ce> {
    let bytes = bincode::serialize(&request)?;
    client
        .write
        .send(Message::Binary(bytes.into()))
        .map_err(|_| Ce::ChannelSend)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let node = TestNode::start().await;
        let mut client = connect(node.port()).await;
        let wasm = load_wasm();
        let _key = deploy(&mut client, &wasm).await;
    }
}
