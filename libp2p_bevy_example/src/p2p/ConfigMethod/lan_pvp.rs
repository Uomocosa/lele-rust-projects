use crate::p2p::Config;

pub fn lan_pvp() -> Config {
    Config::pvp()
}

#[cfg(test)]
mod tests {
    use crate::p2p::Config;

    #[test]
    fn test_usage() {
        let config = Config::lan_pvp();
        assert!(!config.auto_accept_join);
        assert_eq!(config.max_players, Some(2));
    }
}
