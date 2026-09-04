use freenet_stdlib::client_api::ClientRequest;
use futures_util::SinkExt;
use tokio_tungstenite::tungstenite::Message;

use crate::freenet_client;

/// # Errors
/// Returns error if serialization or websocket send fails.
pub async fn send(
    client: &mut freenet_client::FreenetClient,
    req: ClientRequest<'_>,
) -> Result<(), String> {
    let bytes = bincode::serialize(&req).map_err(|e| e.to_string())?;
    client
        .0
        .send(Message::Binary(bytes.into()))
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_usage() {
        let _ = stringify!(send);
    }
}
