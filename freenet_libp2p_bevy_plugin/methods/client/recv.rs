use freenet_stdlib::client_api::HostResponse;

use crate::discovery;

pub async fn recv(client: &mut discovery::Client) -> Result<HostResponse, discovery::Error> {
    match client.read.recv().await {
        Some(Ok(response)) => Ok(response),
        Some(Err(e)) => Err(discovery::Error::from(e)),
        None => Err(discovery::Error::Disconnected),
    }
}

#[cfg(test)]
mod tests {
    use super::recv;
    use crate::discovery;

    #[tokio::test]
    async fn test_usage() {
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        drop(read_tx);
        let mut client = discovery::Client { write, read };
        assert!(recv(&mut client).await.is_err());
    }
}
