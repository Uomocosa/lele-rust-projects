use tokio_tungstenite::tungstenite::Message;

use freenet_stdlib::client_api::{ClientError, HostResponse};

use super::freenet_client_connect;
use super::freenet_client_recv;
use super::freenet_client_recv_response;
use super::freenet_client_recv_response_timeout;
use super::freenet_client_recv_timeout;
use super::freenet_client_send;
use super::freenet_client_wait_ready;
use crate::freenet;

pub struct FreenetClient {
    pub(crate) write: tokio::sync::mpsc::UnboundedSender<Message>,
    pub(crate) read: tokio::sync::mpsc::UnboundedReceiver<Result<HostResponse, ClientError>>,
}

#[rustfmt::skip]
impl FreenetClient {
    pub async fn connect(host: &str, port: u16) -> Result<Self, freenet::FreenetConnectionError> {
        freenet_client_connect::connect(host, port).await
    }
    pub async fn send(&self, request: freenet_stdlib::client_api::ClientRequest<'_>) -> Result<(), freenet::FreenetConnectionError> {
        freenet_client_send::send(self, request).await
    }
    pub async fn recv(&mut self) -> Result<HostResponse, freenet::FreenetConnectionError> {
        freenet_client_recv::recv(self).await
    }
    pub async fn recv_timeout(&mut self, timeout: std::time::Duration) -> Option<Result<HostResponse, freenet::FreenetConnectionError>> {
        freenet_client_recv_timeout::recv_timeout(self, timeout).await
    }
    pub async fn recv_response(&mut self) -> Result<HostResponse, freenet::FreenetConnectionError> {
        freenet_client_recv_response::recv_response(self).await
    }
    pub async fn recv_response_timeout(&mut self, timeout: std::time::Duration) -> Option<Result<HostResponse, freenet::FreenetConnectionError>> {
        freenet_client_recv_response_timeout::recv_response_timeout(self, timeout).await
    }
    pub async fn wait_ready(&mut self, min_active_connections: usize, timeout: std::time::Duration) -> Result<(), freenet::FreenetConnectionError> {
        freenet_client_wait_ready::wait_ready(self, min_active_connections, timeout).await
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {}
}
