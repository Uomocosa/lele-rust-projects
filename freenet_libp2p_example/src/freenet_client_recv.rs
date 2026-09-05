use freenet_stdlib::client_api::HostResponse;
use futures_util::StreamExt;
use tokio_tungstenite::tungstenite::Message;

use crate::freenet_client::FreenetClient;

/// # Errors
/// Returns error if stream closed or message invalid.
pub async fn recv(client: &mut FreenetClient) -> Result<HostResponse, String> {
    loop {
        let msg = client
            .0
            .next()
            .await
            .ok_or_else(|| "stream closed".to_string())?
            .map_err(|e| e.to_string())?;
        if let Message::Binary(b) = msg {
            let decoded: Result<HostResponse, freenet_stdlib::client_api::ClientError> =
                bincode::deserialize(&b).map_err(|e| e.to_string())?;
            match decoded {
                Ok(r) => return Ok(r),
                Err(e) => return Err(e.to_string()),
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_usage() {
        let _ = stringify!(recv);
    }
}
