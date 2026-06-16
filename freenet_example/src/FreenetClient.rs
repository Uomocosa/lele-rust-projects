use tokio_tungstenite::tungstenite::Message;

use freenet_stdlib::client_api::{ClientError, HostResponse};

pub struct FreenetClient {
    pub(crate) write: tokio::sync::mpsc::UnboundedSender<Message>,
    pub(crate) read: tokio::sync::mpsc::UnboundedReceiver<Result<HostResponse, ClientError>>,
}

#[rustfmt::skip]
impl FreenetClient {
    pub async fn connect(host: &str, port: u16) -> Result<Self, crate::ClientError> {
        crate::FreenetClientMethod::connect(host, port).await
    }
    pub async fn send(&self, request: freenet_stdlib::client_api::ClientRequest<'_>) -> Result<(), crate::ClientError> {
        crate::FreenetClientMethod::send(self, request).await
    }
    pub async fn recv(&mut self) -> Result<HostResponse, crate::ClientError> {
        crate::FreenetClientMethod::recv(self).await
    }
    pub async fn recv_timeout(&mut self, timeout: std::time::Duration) -> Option<Result<HostResponse, crate::ClientError>> {
        crate::FreenetClientMethod::recv_timeout(self, timeout).await
    }
    pub async fn recv_response(&mut self) -> Result<HostResponse, crate::ClientError> {
        crate::FreenetClientMethod::recv_response(self).await
    }
    pub async fn recv_response_timeout(&mut self, timeout: std::time::Duration) -> Option<Result<HostResponse, crate::ClientError>> {
        crate::FreenetClientMethod::recv_response_timeout(self, timeout).await
    }
}
