use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClickerError {
    #[error("client error: {0}")]
    Client(#[from] crate::ClientError),
    #[error("timeout")]
    Timeout,
}

#[cfg(test)]
mod tests {
    use super::ClickerError;
    use crate::ClientError;

    #[test]
    fn test_usage() {
        let e = ClickerError::Timeout;
        assert_eq!(e.to_string(), "timeout");
        let e2: ClickerError = ClientError::ConnectionTimeout.into();
        assert!(matches!(
            e2,
            ClickerError::Client(ClientError::ConnectionTimeout)
        ));
    }
}
