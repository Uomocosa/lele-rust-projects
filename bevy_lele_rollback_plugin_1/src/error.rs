use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("prediction would exceed the configured lookahead limit of {limit}")]
    PredictionLookaheadExceeded { limit: u64 },
    #[error("no committed frame is available yet")]
    NoCommittedFrame,
}

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn test_usage() {
        assert_eq!(
            Error::PredictionLookaheadExceeded { limit: 4 }.to_string(),
            "prediction would exceed the configured lookahead limit of 4"
        );
        assert_eq!(
            Error::NoCommittedFrame.to_string(),
            "no committed frame is available yet"
        );
    }
}
