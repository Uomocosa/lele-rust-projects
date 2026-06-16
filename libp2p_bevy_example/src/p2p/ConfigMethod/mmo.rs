use crate::p2p::Config;

pub fn mmo() -> Config {
    Config {
        enable_mdns: false,
        enable_manual_dial: true,
        heartbeat_interval_ms: 5000,
        connection_timeout_ms: 60000,
        auto_accept_join: true,
        max_players: None,
    }
}

#[cfg(test)]
mod tests {
    use crate::p2p::Config;

    #[test]
    fn test_usage() {
        let config = Config::mmo();
        assert!(!config.enable_mdns);
        assert!(config.auto_accept_join);
    }
}
