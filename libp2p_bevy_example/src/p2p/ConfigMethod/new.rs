use crate::p2p::Config;

pub fn new() -> Config {
    Config::default()
}

#[cfg(test)]
mod tests {
    use crate::p2p::Config;

    #[test]
    fn test_usage() {
        let config = Config::new();
        assert!(config.enable_mdns);
    }
}
