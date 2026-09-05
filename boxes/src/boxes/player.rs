use bevy::prelude::Component;
use bevy::prelude::{Commands, Entity, Vec2};
use derive_more::Deref;

use super::player_spawn_box;
use freenet_libp2p_bevy_plugin::net_id;

#[derive(Component, Debug, Clone, Copy, Deref)]
pub struct Player(pub net_id::NetworkId);

#[rustfmt::skip]
impl Player {
    pub fn spawn_box(self, commands: &mut Commands, position: Vec2, is_local: bool) -> Entity { player_spawn_box::spawn_box(self, commands, position, is_local) }
}

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
