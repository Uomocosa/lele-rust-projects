use bevy::prelude::Component;
use derive_more::Deref;

use freenet_libp2p_bevy_plugin::net_id;

#[derive(Component, Debug, Clone, Copy, Deref)]
pub struct Owner(pub net_id::NetworkId);

#[cfg(test)]
mod tests {
    use super::Owner;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let owner = Owner(net_id::NetworkId(7));
        assert_eq!(*owner, net_id::NetworkId(7));
        assert_eq!(**owner, 7);
    }
}
