use freenet_stdlib::client_api::HostResponse;

pub async fn recv(client: &mut crate::FreenetClient) -> Result<HostResponse, crate::ClientError> {
    match client.read.recv().await {
        Some(Ok(response)) => Ok(response),
        Some(Err(e)) => Err(crate::ClientError::FreenetClient(e)),
        None => Err(crate::ClientError::Disconnected),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {}
}
