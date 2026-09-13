use super::client_connect;
use super::client_recv;
use super::client_recv_response;
use super::client_recv_response_timeout;
use super::client_send;
use crate::discovery;

pub struct Client {
    pub(crate) write: tokio::sync::mpsc::UnboundedSender<tokio_tungstenite::tungstenite::Message>,
    pub(crate) read: tokio::sync::mpsc::UnboundedReceiver<
        Result<freenet_stdlib::client_api::HostResponse, freenet_stdlib::client_api::ClientError>,
    >,
}

impl Client {
    /// # Errors
    /// Returns `Error` if the WebSocket handshake or connection fails.
    pub async fn connect(host: &str, port: u16) -> Result<Self, discovery::Error> {
        client_connect::connect(host, port).await
    }
    /// # Errors
    /// Returns `Error::Disconnected` if the channel closes.
    pub async fn recv(
        &mut self,
    ) -> Result<freenet_stdlib::client_api::HostResponse, discovery::Error> {
        client_recv::recv(self).await
    }
    /// # Errors
    /// Returns `Error::Disconnected` if the channel closes.
    pub async fn recv_response(
        &mut self,
    ) -> Result<freenet_stdlib::client_api::HostResponse, discovery::Error> {
        client_recv_response::recv_response(self).await
    }
    pub async fn recv_response_timeout(
        &mut self,
        timeout: std::time::Duration,
    ) -> Option<Result<freenet_stdlib::client_api::HostResponse, discovery::Error>> {
        client_recv_response_timeout::recv_response_timeout(self, timeout).await
    }
}

#[rustfmt::skip]
impl Client {
    /// # Errors
    /// Returns `Error` if serialization fails or the channel is closed.
    pub fn send(&self, request: &freenet_stdlib::client_api::ClientRequest<'_>) -> Result<(), discovery::Error> { client_send::send(self, request) }
}
// no test_usage necessary
