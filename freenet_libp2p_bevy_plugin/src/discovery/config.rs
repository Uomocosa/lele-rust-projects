use crate::p2p;

use super::constants;
use super::params::node_mode::NodeMode;

pub struct Config {
    pub namespace: String,
    pub lobby: Option<String>,
    pub params_override: Option<String>,
    pub node: NodeMode,
    pub transport: p2p::TransportMode,
    pub since_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            namespace: constants::DEFAULT_NAMESPACE.to_string(),
            lobby: None,
            params_override: None,
            node: NodeMode::Embedded,
            transport: p2p::TransportMode::Both,
            since_secs: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn test_usage() {
        let config = Config::default();
        assert_eq!(config.namespace, "blackboard-v1");
        assert!(config.lobby.is_none());
    }
}
