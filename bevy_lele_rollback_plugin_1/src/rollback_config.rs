pub struct RollbackConfig {
    pub max_committed_frames: usize,
    pub prediction_lookahead: u64,
}

impl Default for RollbackConfig {
    fn default() -> Self {
        Self {
            max_committed_frames: 64,
            prediction_lookahead: 8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RollbackConfig;

    #[test]
    fn test_usage() {
        let config = RollbackConfig::default();
        assert_eq!(config.max_committed_frames, 64);
        assert_eq!(config.prediction_lookahead, 8);
    }
}
