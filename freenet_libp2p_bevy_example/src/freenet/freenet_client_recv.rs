use freenet_stdlib::client_api::HostResponse;

use crate::freenet;

pub async fn recv(
    client: &mut freenet::FreenetClient,
) -> Result<HostResponse, freenet::FreenetConnectionError> {
    match client.read.recv().await {
        Some(Ok(response)) => Ok(response),
        Some(Err(e)) => Err(freenet::FreenetConnectionError::FreenetClient(e)),
        None => Err(freenet::FreenetConnectionError::Disconnected),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {}
}
