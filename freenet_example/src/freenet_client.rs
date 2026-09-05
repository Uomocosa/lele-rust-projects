use crate::client_error::ClientError;
use crate::freenet_client_method;
use tokio_tungstenite::tungstenite::Message;

use freenet_stdlib::client_api::{ClientError as StdClientError, HostResponse};

pub struct FreenetClient {
    pub(crate) write: tokio::sync::mpsc::UnboundedSender<Message>,
    pub(crate) read: tokio::sync::mpsc::UnboundedReceiver<Result<HostResponse, StdClientError>>,
}

impl FreenetClient {
    /// # Errors
    /// Returns `ClientError` if the WebSocket handshake or connection fails.
    pub async fn connect(host: &str, port: u16) -> Result<Self, ClientError> {
        freenet_client_method::connect(host, port).await
    }
    /// # Errors
    /// Returns `ClientError` if serialization fails or the channel is closed.
    pub async fn send(
        &self,
        request: freenet_stdlib::client_api::ClientRequest<'_>,
    ) -> Result<(), ClientError> {
        freenet_client_method::send(self, request).await
    }
    /// # Errors
    /// Returns `ClientError::Disconnected` if the channel closes or `ClientError::FreenetClient` on node errors.
    pub async fn recv(&mut self) -> Result<HostResponse, ClientError> {
        freenet_client_method::recv(self).await
    }
    pub async fn recv_timeout(
        &mut self,
        timeout: std::time::Duration,
    ) -> Option<Result<HostResponse, ClientError>> {
        freenet_client_method::recv_timeout(self, timeout).await
    }
    /// # Errors
    /// Returns `ClientError::Disconnected` or `ClientError::FreenetClient` on node errors.
    pub async fn recv_response(&mut self) -> Result<HostResponse, ClientError> {
        freenet_client_method::recv_response(self).await
    }
    pub async fn recv_response_timeout(
        &mut self,
        timeout: std::time::Duration,
    ) -> Option<Result<HostResponse, ClientError>> {
        freenet_client_method::recv_response_timeout(self, timeout).await
    }
}

// no test_usage necessary — exercised via integration tests
