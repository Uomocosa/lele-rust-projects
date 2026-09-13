use freenet_stdlib::client_api::HostResponse;

use crate::discovery;

pub async fn recv(client: &mut discovery::Client) -> Result<HostResponse, discovery::Error> {
    match client.read.recv().await {
        Some(Ok(response)) => Ok(response),
        Some(Err(e)) => Err(discovery::Error::from(e)),
        None => Err(discovery::Error::Disconnected),
    }
}
// no test_usage necessary
