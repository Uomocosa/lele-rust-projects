use super::config::Config;
use freenet_libp2p_bevy_plugin::net_id;

pub fn new(own_id: net_id::NetworkId) -> Config {
    Config(own_id)
}

#[cfg(test)]
mod tests {
    use super::new;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let config = new(net_id::NetworkId(7));
        assert_eq!(*config, net_id::NetworkId(7));
    }
}
