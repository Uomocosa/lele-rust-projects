use thiserror::Error;

#[derive(Error, Debug)]
pub enum FreenetConnectionError {
    #[error("connection timed out")]
    ConnectionTimeout,
    #[error("disconnected from node")]
    Disconnected,
    #[error("websocket error: {0}")]
    WebSocket(Box<tokio_tungstenite::tungstenite::Error>),
    #[error("serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("http error: {0}")]
    Http(#[from] http::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("channel send error")]
    ChannelSend,
    #[error("response timeout")]
    ResponseTimeout,
    #[error("contract not found")]
    ContractNotFound,
    #[error("freenet client error: {0}")]
    FreenetClient(#[from] freenet_stdlib::client_api::ClientError),
    #[error("send error")]
    SendError,
    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),
}

#[rustfmt::skip]
impl From<http::uri::InvalidUri> for FreenetConnectionError {
    fn from(e: http::uri::InvalidUri) -> Self {
        FreenetConnectionError::Http(http::Error::from(e))
    }
}

#[rustfmt::skip]
impl From<tokio_tungstenite::tungstenite::Error> for FreenetConnectionError {
    fn from(e: tokio_tungstenite::tungstenite::Error) -> Self {
        FreenetConnectionError::WebSocket(Box::new(e))
    }
}

#[cfg(test)]
mod tests {
    use super::FreenetConnectionError;

    #[test]
    fn test_usage() {
        let e = FreenetConnectionError::ConnectionTimeout;
        assert_eq!(e.to_string(), "connection timed out");
    }
}
