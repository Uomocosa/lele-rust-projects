use bevy::prelude::*;
use clap::Parser;
use freenet_libp2p_bevy_plugin_lib::net_id::NetworkId;
use freenet_libp2p_bevy_plugin_lib::p2p;
use freenet_libp2p_bevy_plugin_lib::plugin::{Config, P2PPlugin};

use freenet_libp2p_bevy_example_lib::board::SharedBlackboardPlugin;
use freenet_libp2p_bevy_example_lib::board::Stamp;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "blackboard-v1")]
    namespace: String,
}

fn main() {
    let args = Args::parse();
    let _ = args.namespace;

    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Command<Stamp>>();
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Event<Stamp>>();
    let _ = (cmd_rx, event_tx);

    let own_id = NetworkId(1);

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(P2PPlugin(Config::<Stamp>::new(own_id, cmd_tx, event_rx)))
        .add_plugins(SharedBlackboardPlugin)
        .run();
}
