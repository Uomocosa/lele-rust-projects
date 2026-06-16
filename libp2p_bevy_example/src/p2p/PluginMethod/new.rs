use crate::p2p::Config;
use crate::p2p::Plugin;

pub fn new(config: Config) -> Plugin {
    Plugin { config }
}

#[cfg(test)]
mod tests {
    use crate::p2p::Config;
    use crate::p2p::Plugin;

    #[test]
    fn test_usage() {
        let config = Config::default();
        let plugin = Plugin::new(config);
        assert!(plugin.config.enable_mdns);
        assert!(plugin.config.auto_accept_join);
    }
}
