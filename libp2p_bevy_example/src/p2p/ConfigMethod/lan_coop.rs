use crate::p2p::Config;

pub fn lan_coop() -> Config {
    Config::coop()
}

#[cfg(test)]
mod tests {
    use crate::p2p::Config;

    #[test]
    fn test_usage() {
        let config = Config::lan_coop();
        assert!(config.enable_mdns);
    }
}
