use bevy::prelude::Component;
use derive_more::Deref;

use freenet_libp2p_bevy_plugin::net_id;

#[derive(Component, Debug, Clone, Copy, Deref)]
pub struct Player(pub net_id::NetworkId);

#[cfg(test)]
mod tests {
    use super::Player;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let player = Player(net_id::NetworkId(7));
        assert_eq!(*player, net_id::NetworkId(7));
        assert_eq!(**player, 7);
    }
}
