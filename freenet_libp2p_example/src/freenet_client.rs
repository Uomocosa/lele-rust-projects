use derive_more::Deref;

use crate::freenet_client_connect;
use crate::freenet_client_recv;
use crate::freenet_client_recv_with_timeout;
use crate::freenet_client_send;

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

#[derive(Deref)]
pub struct FreenetClient(pub WsStream);

impl FreenetClient {
    /// # Errors
    /// Returns error if websocket connection fails.
    pub async fn connect(host: &str, port: u16) -> Result<Self, String> {
        freenet_client_connect::connect(host, port).await
    }

    /// # Errors
    /// Returns error if serialization or send fails.
    pub async fn send(
        &mut self,
        req: freenet_stdlib::client_api::ClientRequest<'_>,
    ) -> Result<(), String> {
        freenet_client_send::send(self, req).await
    }

    /// # Errors
    /// Returns error if stream closed or deserialization fails.
    pub async fn recv(&mut self) -> Result<freenet_stdlib::client_api::HostResponse, String> {
        freenet_client_recv::recv(self).await
    }

    #[must_use]
    pub async fn recv_with_timeout(
        &mut self,
        dur: std::time::Duration,
    ) -> Option<Result<freenet_stdlib::client_api::HostResponse, String>> {
        freenet_client_recv_with_timeout::recv_with_timeout(self, dur).await
    }
}

// no test_usage necessary
