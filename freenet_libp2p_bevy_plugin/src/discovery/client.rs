use std::time::Duration;

use atomic_delegate_macros::atomic_delegate;
use freenet_stdlib::client_api::ClientError;
use freenet_stdlib::client_api::ClientRequest;
use freenet_stdlib::client_api::HostResponse;
use tokio_tungstenite::tungstenite::Message;

use crate::discovery;

pub struct Client {
    pub(crate) write: tokio::sync::mpsc::UnboundedSender<Message>,
    pub(crate) read: tokio::sync::mpsc::UnboundedReceiver<Result<HostResponse, ClientError>>,
}

#[atomic_delegate]
impl Client {
    pub async fn connect(host: &str, port: u16) -> Result<Self, discovery::Error> {}
    pub fn send(&self, request: &ClientRequest<'_>) -> Result<(), discovery::Error> {}
    pub async fn recv(&mut self) -> Result<HostResponse, discovery::Error> {}
    pub async fn recv_response(&mut self) -> Result<HostResponse, discovery::Error> {}
    pub async fn recv_response_timeout(
        &mut self,
        timeout: Duration,
    ) -> Option<Result<HostResponse, discovery::Error>> {
    }
}
// no test_usage necessary
