use thiserror::Error;

use crate::freenet;

#[derive(Error, Debug)]
pub enum Error {
    #[error("client error: {0}")]
    Client(#[from] freenet::FreenetConnectionError),
    #[error("timeout")]
    Timeout,
}

#[cfg(test)]
mod tests {
    use super::Error;
    use crate::freenet;

    #[test]
    fn test_usage() {
        let e = Error::Timeout;
        assert_eq!(e.to_string(), "timeout");
        let e2: Error = freenet::FreenetConnectionError::ConnectionTimeout.into();
        assert!(matches!(
            e2,
            Error::Client(freenet::FreenetConnectionError::ConnectionTimeout)
        ));
    }
}
