use bevy::prelude::*;

use crate::p2p;

pub fn build(plugin: &p2p::Plugin, app: &mut App) {
    let cmd_tx = plugin.cmd_tx.clone();
    let event_rx = plugin.take_event_rx();

    let dialed: p2p::DialedPeers = Default::default();

    app.insert_resource(p2p::P2pCommands(cmd_tx))
        .insert_resource(p2p::P2pEvents(event_rx))
        .insert_resource(dialed)
        .insert_resource(Time::<Fixed>::from_seconds(
            p2p::constants::FIXED_STEP as f64,
        ))
        .add_systems(Update, p2p::bevy_systems::dial_roster_peers);
}
// no test_usage necessary — exercised by plugin.rs test_usage
