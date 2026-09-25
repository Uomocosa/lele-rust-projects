use std::time::Duration;

use freenet_stdlib::client_api::HostResponse;

use crate::discovery;

pub async fn recv_response_timeout(
    client: &mut discovery::link::Client,
    timeout: Duration,
) -> Option<Result<HostResponse, discovery::Error>> {
    tokio::time::timeout(timeout, client.recv_response())
        .await
        .ok()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::recv_response_timeout;
    use crate::discovery;

    #[tokio::test]
    async fn test_usage() {
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        drop(read_tx);
        let mut client = discovery::link::Client { write, read };
        let result = recv_response_timeout(&mut client, Duration::from_millis(1)).await;
        assert!(matches!(result, Some(Err(_))));
    }
}
