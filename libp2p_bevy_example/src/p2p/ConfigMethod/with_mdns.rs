use crate::p2p::Config;

pub fn with_mdns(mut cfg: Config, enabled: bool) -> Config {
    cfg.enable_mdns = enabled;
    cfg
}

#[cfg(test)]
mod tests {
    use crate::p2p::Config;

    #[test]
    fn test_usage() {
        let config = Config::default().with_mdns(false);
        assert!(!config.enable_mdns);
    }
}
