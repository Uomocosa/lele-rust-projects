use bevy::prelude::*;
use bevy::window::CursorOptions;
use clap::Parser;
use clicker_lib::clicker;
use freenet_libp2p_bevy_plugin::net_id;
use freenet_libp2p_bevy_plugin::p2p;
use freenet_libp2p_bevy_plugin::plugin::{Config, P2PPlugin};
use freenet_libp2p_bevy_plugin::roster;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "blackboard-v1")]
    namespace: String,
    #[arg(long, default_value = "default")]
    lobby: String,
    #[arg(long)]
    create_lobby: bool,
    #[arg(long)]
    list_lobbies: bool,
    #[arg(long)]
    own_id: Option<u64>,
    #[arg(long, default_value_t = 0)]
    instance_tag: u32,
    #[arg(long)]
    dial: Vec<String>,
    #[arg(long, default_value_t = false)]
    auto_click: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let own_id = args
        .own_id
        .unwrap_or_else(|| u64::from(args.instance_tag).wrapping_add(1));
    let (cmd_tx, cmd_rx) =
        tokio::sync::mpsc::unbounded_channel::<p2p::Command<clicker::CursorMsg>>();
    let (event_tx, event_rx) =
        tokio::sync::mpsc::unbounded_channel::<p2p::Event<clicker::CursorMsg>>();
    let _runner = p2p::spawn_runner(cmd_rx, event_tx);
    for addr in &args.dial {
        cmd_tx
            .send(p2p::Command::Dial {
                peer_id: String::new(),
                addrs: vec![addr.clone()],
            })
            .ok();
    }
    if args.create_lobby {
        tracing::info!("lobby {} created (cap {})", args.lobby, clicker::LOBBY_CAP);
    }
    App::new()
        .insert_resource(clicker::InstanceInfo {
            namespace: args.namespace,
            instance_tag: args.instance_tag,
            own_id: net_id::NetworkId(own_id),
        })
        .insert_resource(clicker::ActiveLobby(args.lobby.clone()))
        .insert_resource(clicker::GlobalCounter::default())
        .insert_resource(clicker::AutoClick(args.auto_click))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: format!("clicker-{} [{}]", args.instance_tag, args.lobby),
                resolution: (800, 450).into(),
                ..default()
            }),
            primary_cursor_options: Some(CursorOptions {
                visible: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(P2PPlugin(Config::<clicker::CursorMsg>::new(
            net_id::NetworkId(own_id),
            cmd_tx,
            event_rx,
        )))
        .insert_resource(roster::Lobby(args.lobby))
        .add_plugins(clicker::Plugin)
        .run();
}
