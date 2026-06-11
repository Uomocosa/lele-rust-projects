use std::time::Duration;

use freenet_stdlib::client_api::{ClientError, ClientRequest, HostResponse};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::info;

pub struct FreenetClient {
    write: tokio::sync::mpsc::UnboundedSender<Message>,
    read: tokio::sync::mpsc::UnboundedReceiver<Result<HostResponse, ClientError>>,
}

impl FreenetClient {
    pub async fn connect(host: &str, port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let url = format!("ws://{host}:{port}/v1/contract/command?encodingProtocol=native");
        info!(target: "freenet_example", url = %url, "connecting to freenet node");

        use tokio_tungstenite::tungstenite::client::IntoClientRequest;
        let mut request = url.into_client_request()?;
        request.headers_mut().insert(
            "encoding-protocol",
            http::HeaderValue::from_static("native"),
        );
        let ws = tokio::time::timeout(Duration::from_secs(5), connect_async(request));
        let (ws_stream, _) = ws.await.map_err(|_| "connection timed out after 5s")??;
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

        Ok(Self {
            write: write_tx,
            read: read_rx,
        })
    }

    pub async fn send(&self, request: ClientRequest<'_>) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = bincode::serialize(&request)?;
        self.write.send(Message::Binary(bytes.into()))?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<HostResponse, Box<dyn std::error::Error>> {
        match self.read.recv().await {
            Some(Ok(response)) => Ok(response),
            Some(Err(e)) => Err(Box::new(e)),
            None => Err("disconnected".into()),
        }
    }

    pub async fn recv_timeout(
        &mut self,
        timeout: Duration,
    ) -> Option<Result<HostResponse, Box<dyn std::error::Error>>> {
        tokio::time::timeout(timeout, self.recv()).await.ok()
    }
}
