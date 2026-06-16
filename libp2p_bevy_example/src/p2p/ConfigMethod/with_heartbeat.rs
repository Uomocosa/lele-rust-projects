use crate::p2p::Config;

pub fn with_heartbeat(mut cfg: Config, ms: u64) -> Config {
    cfg.heartbeat_interval_ms = ms;
    cfg
}

#[cfg(test)]
mod tests {
    use crate::p2p::Config;

    #[test]
    fn test_usage() {
        let config = Config::default().with_heartbeat(1000);
        assert_eq!(config.heartbeat_interval_ms, 1000);
    }
}
