use bevy::prelude::*;
use libp2p::PeerId;

use crate::p2p::resource::Fake;
use crate::p2p::resource::PeerState;
use crate::p2p::NetworkEvent;

pub fn trigger_fake_player_join(
    fake_network: Res<Fake>,
    mut p2p_state: ResMut<PeerState>,
    mut events: MessageWriter<NetworkEvent>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if !fake_network.enabled {
        return;
    }

    if keyboard.just_pressed(KeyCode::KeyP) {
        let fake_peer = PeerId::random();
        p2p_state.add_connected_peer(fake_peer);
        events.write(NetworkEvent::PeerConnected(fake_peer));
        tracing::info!("Fake: Simulated player join: {}", fake_peer);
    }
}

#[cfg(test)]
mod tests {
    use crate::p2p::resource::Fake;
    use crate::p2p::resource::PeerState;
    use crate::p2p::Config;
    use crate::p2p::NetworkEvent;
    use bevy::input::ButtonInput;
    use bevy::prelude::*;
    use libp2p::PeerId;

    #[test]
    fn test_usage() {
        let fake = Fake::new();
        let p2p_state = PeerState::new(&Config::default(), PeerId::random());
        let mut keyboard = ButtonInput::<KeyCode>::default();

        keyboard.press(KeyCode::KeyP);

        let mut app = App::new();
        app.insert_resource(fake);
        app.insert_resource(p2p_state);
        app.insert_resource(keyboard);
        app.add_message::<NetworkEvent>();
        app.add_systems(Update, crate::p2p::system::trigger_fake_player_join);
        app.update();

        let p2p_state = app.world().resource::<PeerState>();
        assert!(
            !p2p_state.connected_peers.is_empty(),
            "Should have connected peer after trigger"
        );
    }
}
