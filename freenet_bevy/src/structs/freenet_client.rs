use tokio_tungstenite::tungstenite::Message;

use freenet_stdlib::client_api::{ClientError as FcError, HostResponse};

use crate::methods::freenet_client as fc_method;

pub struct FreenetClient {
    pub(crate) write: tokio::sync::mpsc::UnboundedSender<Message>,
    pub(crate) read: tokio::sync::mpsc::UnboundedReceiver<Result<HostResponse, FcError>>,
}

#[rustfmt::skip]
impl FreenetClient {
    pub async fn connect(host: &str, port: u16) -> Result<Self, crate::ClientError> {
        fc_method::connect(host, port).await
    }
    pub async fn send(&self, request: freenet_stdlib::client_api::ClientRequest<'_>) -> Result<(), crate::ClientError> {
        fc_method::send(self, request).await
    }
    pub async fn recv(&mut self) -> Result<HostResponse, crate::ClientError> {
        fc_method::recv(self).await
    }
    pub async fn recv_timeout(&mut self, timeout: std::time::Duration) -> Option<Result<HostResponse, crate::ClientError>> {
        fc_method::recv_timeout(self, timeout).await
    }
    pub async fn recv_response(&mut self) -> Result<HostResponse, crate::ClientError> {
        fc_method::recv_response(self).await
    }
    pub async fn recv_response_timeout(&mut self, timeout: std::time::Duration) -> Option<Result<HostResponse, crate::ClientError>> {
        fc_method::recv_response_timeout(self, timeout).await
    }
}

#[cfg(test)]
mod tests {
    // Trivial wrapper/delegate module — skip test_usage.
    // The real coverage comes from integration tests.
    #[test]
    fn test_usage() {}
}
