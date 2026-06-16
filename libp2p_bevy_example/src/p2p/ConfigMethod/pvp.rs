use crate::p2p::Config;

pub fn pvp() -> Config {
    Config {
        enable_mdns: true,
        enable_manual_dial: true,
        heartbeat_interval_ms: 2000,
        connection_timeout_ms: 15000,
        auto_accept_join: false,
        max_players: Some(2),
    }
}

#[cfg(test)]
mod tests {
    use crate::p2p::Config;

    #[test]
    fn test_usage() {
        let config = Config::pvp();
        assert!(!config.auto_accept_join);
        assert_eq!(config.max_players, Some(2));
    }
}
