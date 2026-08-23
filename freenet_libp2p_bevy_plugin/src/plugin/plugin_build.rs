use bevy::prelude::*;

use crate::net_id;
use crate::p2p;
use crate::plugin;
use crate::roster;

pub fn build<T: p2p::Message>(plugin: &plugin::Plugin<T>, app: &mut App) {
    let cmd_tx = plugin.cmd_tx.clone();
    let event_rx = plugin.take_event_rx();
    let roster_rx = plugin.take_roster_rx();
    let own_id = plugin.own_id;

    app.insert_resource(p2p::P2pCommands::<T>(cmd_tx))
        .insert_resource(p2p::P2pEvents::<T>(event_rx))
        .insert_resource(p2p::DialedPeers::default())
        .insert_resource(p2p::PeerStatus::default())
        .insert_resource(p2p::SnapshotTick::default())
        .insert_resource(net_id::LocalIdentity(own_id))
        .insert_resource(Time::<Fixed>::from_seconds(
            p2p::constants::FIXED_STEP as f64,
        ))
        .insert_resource(roster::Roster::default())
        .insert_resource(roster::RosterEvents(roster_rx))
        .insert_resource(roster::FreenetStatus::default())
        .add_message::<roster::PeerJoined>()
        .add_message::<roster::PeerLeft>()
        .add_message::<p2p::IncomingSnapshot<T>>()
        .add_message::<p2p::Connected>()
        .add_message::<p2p::Disconnected>()
        .add_systems(
            Update,
            (
                roster::bevy_systems::poll_freenet_events,
                roster::bevy_systems::update_status_text,
                p2p::bevy_systems::dial_roster_peers::<T>,
            )
                .chain(),
        )
        .add_systems(FixedUpdate, p2p::bevy_systems::poll_swarm_events::<T>);
}
// no test_usage necessary — exercised by plugin.rs test_usage
