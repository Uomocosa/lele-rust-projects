use crate::client_error;
use crate::freenet_client;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::info;

use freenet_stdlib::client_api::{ClientError, HostResponse};

use client_error::ClientError as Ce;

/// # Errors
/// Returns `ClientError` if the WebSocket connection fails or times out.
pub async fn connect(host: &str, port: u16) -> Result<freenet_client::FreenetClient, Ce> {
    let url = format!("ws://{host}:{port}/v1/contract/command?encodingProtocol=native");
    info!(target: "freenet_example", url = %url, "connecting to freenet node");

    let mut request = url.into_client_request()?;
    request.headers_mut().insert(
        "encoding-protocol",
        http::HeaderValue::from_static("native"),
    );
    let ws = tokio::time::timeout(Duration::from_secs(5), connect_async(request));
    let (ws_stream, _) = ws.await.map_err(|_| Ce::ConnectionTimeout)??;
    let (mut ws_write, mut ws_read) = ws_stream.split();

    let (write_tx, mut write_rx) = tokio::sync::mpsc::unbounded_channel::<Message>();
    let (read_tx, read_rx) =
        tokio::sync::mpsc::unbounded_channel::<Result<HostResponse, ClientError>>();

    tokio::spawn(async move {
        while let Some(msg) = write_rx.recv().await {
            if ws_write.send(msg).await.is_err() {
                break;
            }
        }
    });

    let write_tx_clone = write_tx.clone();
    tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_read.next().await {
            match msg {
                Message::Binary(data) => {
                    match bincode::deserialize::<Result<HostResponse, ClientError>>(&data) {
                        Ok(result) => {
                            let _ = read_tx.send(result);
                        }
                        Err(e) => {
                            tracing::error!(target: "freenet_example", error = %e, len = data.len(), "failed to deserialize host response");
                        }
                    }
                }
                Message::Close(frame) => {
                    tracing::info!(target: "freenet_example", ?frame, "websocket closed");
                    break;
                }
                Message::Ping(data) => {
                    let _ = write_tx_clone.send(Message::Pong(data));
                }
                _ => {}
            }
        }
    });

    Ok(freenet_client::FreenetClient {
        write: write_tx,
        read: read_rx,
    })
}

#[cfg(all(test, feature = "dev"))]
mod tests {
    use crate::testing::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let node = TestNode::start().await.unwrap();
        let client = connect(node.port).await.unwrap();
        client
            .send(freenet_stdlib::client_api::ClientRequest::Disconnect { cause: None })
            .await
            .unwrap();
    }
}
