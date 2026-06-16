use thiserror::Error;

use crate::ClientError;

#[derive(Error, Debug)]
pub enum ClickerError {
    #[error("client error: {0}")]
    Client(#[from] ClientError),
    #[error("contract not found")]
    ContractNotFound,
    #[error("serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("unexpected response: {0:?}")]
    UnexpectedResponse(String),
    #[error("no response within timeout")]
    Timeout,
}
