use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("player is not spawned in the engine")]
    UnknownPlayer,
}

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn test_usage() {
        let err = Error::UnknownPlayer;
        assert!(err.to_string().contains("not spawned"));
        assert_eq!(err, Error::UnknownPlayer);
    }
}
