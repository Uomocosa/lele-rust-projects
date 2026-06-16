use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("connection timed out")]
    ConnectionTimeout,
    #[error("disconnected")]
    Disconnected,
    #[error("websocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("http error: {0}")]
    Http(#[from] http::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("channel send error")]
    ChannelSend,
    #[error("response timed out")]
    ResponseTimeout,
    #[error("contract not found")]
    ContractNotFound,
    #[error("freenet stdlib error: {0}")]
    FreenetClient(#[from] freenet_stdlib::client_api::ClientError),
    #[error("{0}")]
    SendError(String),
    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),
}
