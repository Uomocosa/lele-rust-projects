mod args;
mod dummy;
mod intent;
mod node;
mod status;

use bevy::prelude::*;
use clap::Parser;
use freenet_libp2p_bevy_plugin::{discovery, net_id, plugin};

#[tokio::main]
async fn main() {
    let args = args::Args::parse();
    init_tracing();
    let Ok((_node_guard, ws_port)) = node::bootstrap_node().await else {
        eprintln!("failed to start embedded freenet node");
        return;
    };
    let transport = args::transport_mode(args.transport);
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: format!("lobby-{}", args.username),
            resolution: (960, 540).into(),
            ..default()
        }),
        ..default()
    }));
    app.insert_resource(discovery::FreenetEndpoint(ws_port));
    app.insert_resource(status::Username(args.username.clone()));
    app.insert_resource(intent::Intent {
        action: args.action.clone(),
        sent: false,
    });
    app.add_plugins(plugin::P2PPlugin(plugin::Config::<dummy::Dummy>::new(
        net_id::NetworkId(0),
        transport,
        false,
    )));
    app.add_plugins(discovery::Plugin::new(discovery::Config {
        game_name: discovery::id::GameName("lobby_room_example".to_string()),
        token: args.token.clone().map_or_else(
            || freenet_libp2p_bevy_plugin::game_token!(),
            discovery::id::GameToken,
        ),
        timing: discovery::Timing::default(),
        transport,
        capacity: 8,
    }));
    app.add_systems(Startup, status::setup_ui);
    app.add_systems(
        Update,
        (intent::send_once, intent::log_joined, status::log_tick),
    );
    app.run();
}

// needed helper: stdout tracing so the run log captures every marker line
fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(false)
        .init();
}
