use crate::client_error;
use crate::freenet_client_method;
use tokio_tungstenite::tungstenite::Message;

use freenet_stdlib::client_api::{ClientError, HostResponse};

pub struct FreenetClient {
    pub(crate) write: tokio::sync::mpsc::UnboundedSender<Message>,
    pub(crate) read: tokio::sync::mpsc::UnboundedReceiver<Result<HostResponse, ClientError>>,
}

#[rustfmt::skip]
impl FreenetClient {
    pub async fn connect(host: &str, port: u16) -> Result<Self, client_error::ClientError> {
        freenet_client_method::connect(host, port).await
    }
    pub async fn send(&self, request: freenet_stdlib::client_api::ClientRequest<'_>) -> Result<(), client_error::ClientError> {
        freenet_client_method::send(self, request).await
    }
    pub async fn recv(&mut self) -> Result<HostResponse, client_error::ClientError> {
        freenet_client_method::recv(self).await
    }
    pub async fn recv_timeout(&mut self, timeout: std::time::Duration) -> Option<Result<HostResponse, client_error::ClientError>> {
        freenet_client_method::recv_timeout(self, timeout).await
    }
    pub async fn recv_response(&mut self) -> Result<HostResponse, client_error::ClientError> {
        freenet_client_method::recv_response(self).await
    }
    pub async fn recv_response_timeout(&mut self, timeout: std::time::Duration) -> Option<Result<HostResponse, client_error::ClientError>> {
        freenet_client_method::recv_response_timeout(self, timeout).await
    }
}

// no test_usage necessary — exercised via integration tests
