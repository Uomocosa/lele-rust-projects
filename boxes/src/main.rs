use bevy::prelude::*;
use boxes_lib::boxes;
use clap::Parser;
use freenet_libp2p_bevy_plugin::net_id::NetworkId;
use freenet_libp2p_bevy_plugin::p2p;
use freenet_libp2p_bevy_plugin::plugin::{Config, P2PPlugin};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "blackboard-v1")]
    namespace: String,
}

fn main() {
    let args = Args::parse();
    let _ = args.namespace;
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Command<boxes::Payload>>();
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Event<boxes::Payload>>();
    let _ = (cmd_rx, event_tx);
    let own_id = NetworkId(1);
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(P2PPlugin(Config::<boxes::Payload>::new(
            own_id, cmd_tx, event_rx,
        )))
        .add_plugins(boxes::Plugin(boxes::Config::new(own_id)))
        .run();
}
