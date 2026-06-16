use crate::p2p::Config;

pub fn coop() -> Config {
    Config {
        enable_mdns: true,
        enable_manual_dial: true,
        heartbeat_interval_ms: 5000,
        connection_timeout_ms: 30000,
        auto_accept_join: true,
        max_players: None,
    }
}

#[cfg(test)]
mod tests {
    use crate::p2p::Config;

    #[test]
    fn test_usage() {
        let config = Config::coop();
        assert!(config.enable_mdns);
        assert!(config.auto_accept_join);
    }
}
