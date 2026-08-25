use thiserror::Error;

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    #[error("reveal does not match the committed hash")]
    RevealMismatch,
    #[error("peer has already sent a commit for this tick")]
    DuplicateCommit,
}

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn test_usage() {
        assert!(Error::RevealMismatch.to_string().contains("committed"));
        assert_eq!(Error::RevealMismatch, Error::RevealMismatch);
    }
}
