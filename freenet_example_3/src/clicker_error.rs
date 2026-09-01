use crate::client_error;
use thiserror::Error;

use client_error::ClientError;

#[derive(Error, Debug)]
pub enum ClickerError {
    #[error("client error: {0}")]
    Client(#[from] ClientError),
    #[error("contract not found")]
    ContractNotFound,
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("unexpected response: {0:?}")]
    UnexpectedResponse(String),
    #[error("no response within timeout")]
    Timeout,
}

#[rustfmt::skip]
impl From<bincode::Error> for ClickerError {
    fn from(e: bincode::Error) -> Self { Self::Serialization(e.to_string()) }
}
