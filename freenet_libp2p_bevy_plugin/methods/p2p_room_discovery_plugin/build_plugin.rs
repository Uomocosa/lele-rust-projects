use std::sync::Mutex;

use bevy::prelude::*;

use crate::discovery;
use crate::net_id;
use crate::p2p;

pub fn build_plugin(p2p_plugin: &discovery::P2PRoomDiscoveryPlugin, app: &mut App) {
    let config = &p2p_plugin.0;
    app.init_state::<discovery::session::RoomState>();
    app.add_message::<discovery::session::RoomRequest>();
    app.init_resource::<discovery::directory::RoomList>();
    app.init_resource::<discovery::session::ActiveRoom>();
    app.init_resource::<discovery::session::SelectedRoom>();
    app.init_resource::<discovery::session::LeftRoom>();
    app.init_resource::<discovery::session::JoinPending>();
    app.init_resource::<discovery::session::JoinGate>();
    app.init_resource::<discovery::session::JoinClock>();
    app.init_resource::<discovery::session::DirectoryLive>();

    let (room_tx, room_rx) = tokio::sync::watch::channel(None);
    let (req_tx, req_rx) = tokio::sync::mpsc::unbounded_channel();
    let (dir_tx, dir_rx) = tokio::sync::mpsc::unbounded_channel();
    let (exp_tx, exp_rx) = tokio::sync::mpsc::unbounded_channel();
    app.insert_resource(discovery::session::RoomRx(Mutex::new(Some(room_rx))));
    app.insert_resource(discovery::session::RoomRequestTx(req_tx));
    app.insert_resource(discovery::session::DirectoryFeed(Mutex::new(Some(dir_rx))));
    app.insert_resource(discovery::session::ExpectedRx(Mutex::new(Some(exp_rx))));

    let tap_rx = app
        .world_mut()
        .get_resource_mut::<p2p::EventTap>()
        .and_then(|tap| tap.take_rx());
    let (ready_rx, observed_rx) = app.world().get_resource::<p2p::Signals>().map_or_else(
        || {
            let (_, ready_rx) = tokio::sync::watch::channel(None);
            let (_, observed_rx) = tokio::sync::watch::channel(None);
            (ready_rx, observed_rx)
        },
        p2p::Signals::subscribe,
    );
    let net_tx = app
        .world()
        .get_resource::<p2p::NetBridge>()
        .map(|bridge| bridge.0.clone());
    let own = app
        .world()
        .get_resource::<net_id::NetworkId>()
        .map_or(discovery::params::PlayerId(0), |id| {
            discovery::params::PlayerId(**id)
        });

    if let (Some(tap_rx), Some(net_tx)) = (tap_rx, net_tx) {
        let run_config = discovery::link::RunConfig {
            net_tx,
            tap_rx,
            ready_rx,
            observed_rx,
            namespace: config.namespace.clone(),
            lobby: config.lobby.clone(),
            params_override: config.params_override.clone(),
            since_secs: config.since_secs,
            own,
            transport: config.transport,
            node: config.node,
            room_tx,
            room_requests: req_rx,
            directory_tx: dir_tx,
            expected_tx: exp_tx,
        };
        tokio::spawn(discovery::link::run(run_config));
    }

    app.add_systems(
        Update,
        (
            discovery::lobby_ui::bevy_systems::poll_directory,
            discovery::lobby_ui::bevy_systems::poll_room,
            discovery::lobby_ui::bevy_systems::poll_expected,
            discovery::lobby_ui::bevy_systems::request_room,
        )
            .chain(),
    );
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;

    use super::build_plugin;
    use crate::discovery;

    #[tokio::test]
    async fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        let plugin = discovery::P2PRoomDiscoveryPlugin::new(discovery::Config::default());
        build_plugin(&plugin, &mut app);
    }
}
