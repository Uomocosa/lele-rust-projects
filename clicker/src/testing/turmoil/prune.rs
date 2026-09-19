use std::collections::HashSet;
use std::hash::BuildHasher;

use bevy::prelude::App;
use freenet_libp2p_bevy_plugin::roster;

use super::key_for_peer::key_for_peer;

pub fn prune<S: BuildHasher>(app: &mut App, peer: &str, dead: &mut HashSet<String, S>) {
    if dead.insert(peer.to_string()) {
        app.world_mut()
            .resource_mut::<roster::Roster>()
            .remove_entry("alpha", key_for_peer(peer));
    }
}

// no test_usage necessary
