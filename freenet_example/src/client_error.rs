use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("connection timed out")]
    ConnectionTimeout,
    #[error("disconnected")]
    Disconnected,
    #[error("websocket error: {0}")]
    WebSocket(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("http error: {0}")]
    Http(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("channel send error")]
    ChannelSend,
    #[error("response timed out")]
    ResponseTimeout,
    #[error("contract not found")]
    ContractNotFound,
    #[error("freenet stdlib error: {0}")]
    FreenetClient(String),
    #[error("{0}")]
    SendError(String),
    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),
    #[error("deadline overflow")]
    DeadlineOverflow,
}

#[rustfmt::skip]
impl From<tokio_tungstenite::tungstenite::Error> for ClientError {
    fn from(e: tokio_tungstenite::tungstenite::Error) -> Self { Self::WebSocket(e.to_string()) }
}

#[rustfmt::skip]
impl From<bincode::Error> for ClientError {
    fn from(e: bincode::Error) -> Self { Self::Serialization(e.to_string()) }
}

#[rustfmt::skip]
impl From<http::Error> for ClientError {
    fn from(e: http::Error) -> Self { Self::Http(e.to_string()) }
}

#[rustfmt::skip]
impl From<std::io::Error> for ClientError {
    fn from(e: std::io::Error) -> Self { Self::Io(e.to_string()) }
}

#[rustfmt::skip]
impl From<freenet_stdlib::client_api::ClientError> for ClientError {
    fn from(e: freenet_stdlib::client_api::ClientError) -> Self { Self::FreenetClient(e.to_string()) }
}
