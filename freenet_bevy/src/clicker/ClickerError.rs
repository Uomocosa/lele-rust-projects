use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClickerError {
    #[error("client error: {0}")]
    Client(#[from] crate::ClientError),
    #[error("timeout")]
    Timeout,
}
