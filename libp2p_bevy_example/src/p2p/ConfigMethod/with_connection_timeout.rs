use crate::p2p::Config;

pub fn with_connection_timeout(mut cfg: Config, ms: u64) -> Config {
    cfg.connection_timeout_ms = ms;
    cfg
}

#[cfg(test)]
mod tests {
    use crate::p2p::Config;

    #[test]
    fn test_usage() {
        let config = Config::default().with_connection_timeout(10000);
        assert_eq!(config.connection_timeout_ms, 10000);
    }
}
