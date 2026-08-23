use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("invalid contract parameters")]
    InvalidParams,
    #[error("roster exceeds max members")]
    TooManyMembers,
    #[error("entry has too many addresses")]
    TooManyAddrs,
    #[error("invalid signature")]
    SignatureInvalid,
    #[error("update rewinds the entry sequence")]
    Rewind,
}

// no test_usage necessary
