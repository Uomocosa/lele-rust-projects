use freenet_stdlib::client_api::{ContractResponse, HostResponse};

use crate::discovery;

pub async fn recv_response(
    client: &mut discovery::link::Client,
) -> Result<HostResponse, discovery::Error> {
    loop {
        match client.recv().await? {
            HostResponse::ContractResponse(ContractResponse::UpdateNotification { .. }) => {}
            other => return Ok(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::recv_response;
    use crate::discovery;

    #[tokio::test]
    async fn test_usage() {
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        drop(read_tx);
        let mut client = discovery::link::Client { write, read };
        assert!(recv_response(&mut client).await.is_err());
    }
}
