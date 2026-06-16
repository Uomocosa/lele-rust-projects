use bevy::prelude::*;
use libp2p::PeerId;

use crate::clicker::component;
use crate::p2p::resource::PeerState;
use crate::p2p::NetworkEvent;

pub fn handle_player_join(
    mut events: MessageReader<NetworkEvent>,
    mut commands: Commands,
    p2p_state: Res<PeerState>,
) {
    for event in events.read() {
        if let NetworkEvent::PeerConnected(peer_id) = event {
            let is_local = *peer_id == p2p_state.local_peer_id;
            spawn_click_button(&mut commands, *peer_id, is_local);
        }
    }
}

pub(crate) fn spawn_click_button(commands: &mut Commands, peer_id: PeerId, is_local: bool) {
    let label = if is_local { "You" } else { "Opponent" };

    commands.spawn((
        component::Owner { peer_id },
        component::ClickCounter { count: 0 },
        component::ClickTarget,
        Text::new(format!("{}: 0", label)),
    ));
}

#[cfg(test)]
mod tests {
    use crate::clicker::system;
    use bevy::prelude::Entity;
    use bevy::prelude::World;
    use libp2p::PeerId;

    #[test]
    fn test_usage() {
        let mut world = World::new();
        let peer_id = PeerId::random();

        {
            let mut commands = world.commands();
            system::handle_player_join::spawn_click_button(&mut commands, peer_id, true);
        }
        world.flush();

        let mut query = world.query::<Entity>();
        assert!(query.single(&world).is_ok());
    }
}
