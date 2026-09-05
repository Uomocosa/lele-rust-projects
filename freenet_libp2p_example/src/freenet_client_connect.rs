use std::time::Duration;

use crate::freenet_client::FreenetClient;

/// # Errors
/// Returns error if connection fails.
pub async fn connect(host: &str, port: u16) -> Result<FreenetClient, String> {
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    let url = format!("ws://{host}:{port}/v1/contract/command?encodingProtocol=native");
    let mut request = url.into_client_request().map_err(|e| e.to_string())?;
    request.headers_mut().insert(
        "encoding-protocol",
        http::HeaderValue::from_static("native"),
    );
    let fut = tokio_tungstenite::connect_async(request);
    let (stream, _) = tokio::time::timeout(Duration::from_secs(5), fut)
        .await
        .map_err(|_| "ws connect timeout".to_string())?
        .map_err(|e| e.to_string())?;
    Ok(FreenetClient(stream))
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_usage() {
        let _ = stringify!(connect);
    }
}
