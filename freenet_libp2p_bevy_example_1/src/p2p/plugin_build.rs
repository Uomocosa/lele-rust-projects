use bevy::prelude::*;

use crate::p2p;

pub fn build(plugin: &p2p::Plugin, app: &mut App) {
    let cmd_tx = plugin.cmd_tx.clone();
    let event_rx = plugin.take_event_rx();

    let dialed: p2p::DialedPeers = Default::default();
    let peer_status: p2p::PeerStatus = Default::default();
    let snapshot_tick: p2p::SnapshotTick = Default::default();

    app.insert_resource(p2p::P2pCommands(cmd_tx))
        .insert_resource(p2p::P2pEvents(event_rx))
        .insert_resource(dialed)
        .insert_resource(peer_status)
        .insert_resource(snapshot_tick)
        .insert_resource(Time::<Fixed>::from_seconds(
            p2p::constants::FIXED_STEP as f64,
        ))
        .add_systems(Update, p2p::bevy_systems::dial_roster_peers)
        .add_systems(
            FixedUpdate,
            (
                p2p::bevy_systems::send_snapshot,
                p2p::bevy_systems::poll_swarm_events,
                p2p::bevy_systems::interpolate_remote_boxes,
            )
                .chain(),
        );
}
// no test_usage necessary — exercised by plugin.rs test_usage
