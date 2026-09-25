use std::sync::Mutex;

use bevy::prelude::*;
use tracing::error;

use crate::discovery;
use crate::p2p;

pub fn build_plugin(plugin: &discovery::Plugin, app: &mut App) {
    app.init_resource::<discovery::Multiplayer>();
    app.add_message::<discovery::Command>();
    app.add_message::<discovery::Event>();
    app.add_systems(
        Update,
        (
            discovery::bevy_systems::drain_multiplayer,
            discovery::bevy_systems::drain_events,
            discovery::bevy_systems::forward_commands,
        ),
    );

    let (multiplayer_tx, multiplayer_rx) = tokio::sync::mpsc::unbounded_channel();
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
    let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();
    app.insert_resource(discovery::MultiplayerFeed(Mutex::new(Some(multiplayer_rx))));
    app.insert_resource(discovery::EventFeed(Mutex::new(Some(event_rx))));
    app.insert_resource(discovery::CommandSender(command_tx));

    let tap = app
        .world_mut()
        .get_resource_mut::<p2p::EventTap>()
        .and_then(|tap| tap.take_rx());
    let (ready, observed) = app.world().get_resource::<p2p::Signals>().map_or_else(
        || {
            let (_, ready) = tokio::sync::watch::channel(None);
            let (_, observed) = tokio::sync::watch::channel(None);
            (ready, observed)
        },
        p2p::Signals::subscribe,
    );
    let net_tx = app
        .world()
        .get_resource::<p2p::NetBridge>()
        .map(|bridge| bridge.0.clone());
    let endpoint = app
        .world()
        .get_resource::<discovery::FreenetEndpoint>()
        .copied();

    let (Some(tap), Some(net_tx), Some(endpoint)) = (tap, net_tx, endpoint) else {
        error!(target: "room_lobby", "discovery: link resources missing, discovery disabled");
        return;
    };

    let run = discovery::session::RunConfig {
        config: plugin.0.clone(),
        endpoint,
        link: discovery::link::NetLink {
            tx: net_tx,
            observed,
        },
        tap,
        ready,
        commands: command_rx,
        multiplayer: multiplayer_tx,
        events: event_tx,
    };
    tokio::spawn(discovery::session::run(run));
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::build_plugin;
    use crate::discovery;

    #[tokio::test]
    async fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let plugin = discovery::Plugin::new(discovery::Config::default());
        build_plugin(&plugin, &mut app);
        assert!(
            app.world()
                .get_resource::<discovery::Multiplayer>()
                .is_some()
        );
    }
}
