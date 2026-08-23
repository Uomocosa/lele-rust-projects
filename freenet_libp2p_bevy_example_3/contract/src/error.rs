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
    #[error("input log rewinds the log sequence")]
    InputLogRewind,
    #[error("input log batch exceeds the ring cap")]
    InputLogTooLarge,
    #[error("input log references an identity not in the roster")]
    IdentityNotInRoster,
    #[error("input log entry is not signed")]
    UnsignedInput,
}

// no test_usage necessary
