use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to build swarm: {0}")]
    Build(String),
    #[error("dial failed: {0}")]
    Dial(String),
    #[error("swarm error: {0}")]
    Swarm(String),
}

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn test_usage() {
        assert_eq!(Error::Dial("x".into()).to_string(), "dial failed: x");
    }
}
