use bevy::prelude::*;
use bevy::window::CursorOptions;
use clap::Parser;
use clicker_lib::clicker;
use clicker_lib::discovery;
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
    contract_params: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let own_id = args
        .own_id
        .unwrap_or_else(|| u64::from(args.instance_tag).wrapping_add(1));
    let params = discovery::resolve_params(
        &args.namespace,
        &args.lobby,
        args.contract_params.as_deref(),
    );
    let (cmd_tx, cmd_rx) =
        tokio::sync::mpsc::unbounded_channel::<p2p::Command<clicker::CursorMsg>>();
    let (raw_tx, raw_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Event<clicker::CursorMsg>>();
    let (bevy_tx, bevy_rx) =
        tokio::sync::mpsc::unbounded_channel::<p2p::Event<clicker::CursorMsg>>();
    let (ready_tx, ready_rx) = tokio::sync::watch::channel::<Option<(String, Vec<String>)>>(None);
    let _runner = p2p::spawn_runner(cmd_rx, raw_tx);
    tokio::spawn(forward_events(raw_rx, bevy_tx, ready_tx));
    tokio::spawn(discovery::run(
        cmd_tx.clone(),
        ready_rx,
        params,
        discovery::PlayerId(own_id),
    ));
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
        .insert_resource(clicker::PendingClicks::default())
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
            bevy_rx,
        )))
        .insert_resource(roster::Lobby(args.lobby))
        .add_plugins(clicker::Plugin)
        .run();
}

async fn forward_events(
    mut raw_rx: tokio::sync::mpsc::UnboundedReceiver<p2p::Event<clicker::CursorMsg>>,
    bevy_tx: tokio::sync::mpsc::UnboundedSender<p2p::Event<clicker::CursorMsg>>,
    ready_tx: tokio::sync::watch::Sender<Option<(String, Vec<String>)>>,
) {
    while let Some(event) = raw_rx.recv().await {
        if let p2p::Event::Ready { peer_id, addrs } = &event {
            ready_tx.send_replace(Some((peer_id.clone(), addrs.clone())));
        }
        if bevy_tx.send(event).is_err() {
            break;
        }
    }
}
