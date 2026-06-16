use freenet_stdlib::client_api::HostResponse;

use crate::ClientError as Ce;

pub async fn recv(client: &mut crate::FreenetClient) -> Result<HostResponse, Ce> {
    match client.read.recv().await {
        Some(Ok(response)) => Ok(response),
        Some(Err(e)) => Err(Ce::FreenetClient(e)),
        None => Err(Ce::Disconnected),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {}
}
