use bevy::prelude::App;
use freenet_libp2p_bevy_plugin::roster;

use super::key_for_peer::key_for_peer;

pub fn link_app(app: &mut App, name: &str) {
    let mut members = app.world_mut().resource_mut::<roster::Roster>();
    for peer in ["peer-1", "peer-2", "peer-3"] {
        if peer != name {
            members.add_entry("alpha".to_string(), key_for_peer(peer), peer.to_string());
        }
    }
}

// no test_usage necessary
