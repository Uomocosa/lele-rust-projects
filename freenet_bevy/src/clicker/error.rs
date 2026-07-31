use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("client error: {0}")]
    Client(#[from] crate::ClientError),
    #[error("timeout")]
    Timeout,
}

#[cfg(test)]
mod tests {
    use super::Error;
    use crate::ClientError;

    #[test]
    fn test_usage() {
        let e = Error::Timeout;
        assert_eq!(e.to_string(), "timeout");
        let e2: Error = ClientError::ConnectionTimeout.into();
        assert!(matches!(e2, Error::Client(ClientError::ConnectionTimeout)));
    }
}
