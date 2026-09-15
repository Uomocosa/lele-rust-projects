use bevy::prelude::*;
use bevy::window::CursorOptions;
use clap::Parser;
use clicker_lib::clicker;
use clicker_lib::discovery;
use clicker_lib::lobby;
use freenet_libp2p_bevy_plugin::net_id;
use freenet_libp2p_bevy_plugin::p2p;
use freenet_libp2p_bevy_plugin::plugin::{Config, P2PPlugin};
use freenet_libp2p_bevy_plugin::roster;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "blackboard-v1")]
    namespace: String,
    #[arg(long)]
    lobby: Option<String>,
    #[arg(long)]
    create_lobby: bool,
    #[arg(long)]
    list_lobbies: bool,
    #[arg(long)]
    since_epoch: Option<u64>,
    #[arg(long)]
    own_id: Option<u64>,
    #[arg(long, default_value_t = 0)]
    instance_tag: u32,
    #[arg(long)]
    contract_params: Option<String>,
    #[arg(long, value_enum, default_value = "both")]
    transport: TransportArg,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum TransportArg {
    Tcp,
    Quic,
    Both,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let own_id = args
        .own_id
        .unwrap_or_else(|| u64::from(args.instance_tag).wrapping_add(1));
    let (cmd_tx, cmd_rx) =
        tokio::sync::mpsc::unbounded_channel::<p2p::Command<clicker::CursorMsg>>();
    let (raw_tx, raw_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Event<clicker::CursorMsg>>();
    let (bevy_tx, bevy_rx) =
        tokio::sync::mpsc::unbounded_channel::<p2p::Event<clicker::CursorMsg>>();
    let (ready_tx, ready_rx) = tokio::sync::watch::channel::<Option<(String, Vec<String>)>>(None);
    let (obs_tx, obs_rx) = tokio::sync::watch::channel::<Option<Vec<String>>>(None);
    let (link_tx, link_rx) = tokio::sync::mpsc::unbounded_channel::<(String, bool)>();
    let (lobby_tx, lobby_rx) =
        tokio::sync::mpsc::unbounded_channel::<p2p::Event<clicker::CursorMsg>>();
    let (room_tx, room_rx) = tokio::sync::watch::channel::<Option<String>>(None);
    let (directory_tx, directory_rx) =
        tokio::sync::mpsc::unbounded_channel::<discovery::DirectoryState>();
    let (request_tx, request_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let lobby_arg = args.lobby.clone();
    if let Some(room) = lobby_arg.as_deref() {
        request_tx.send(room.to_string()).ok();
    }
    let mode = transport_mode(&args.transport);
    let _runner = p2p::spawn_runner(cmd_rx, raw_tx, mode);
    tokio::spawn(forward_events(
        raw_rx, bevy_tx, ready_tx, obs_tx, link_tx, lobby_tx,
    ));
    let namespace_arg = args.namespace.clone();
    let params_arg = args.contract_params.clone();
    let since_secs = args.since_epoch.unwrap_or(0);
    let run_config = discovery::RunConfig {
        cmd_tx: cmd_tx.clone(),
        ready: ready_rx,
        observed: obs_rx,
        links: link_rx,
        lobby_events: lobby_rx,
        namespace: namespace_arg,
        lobby: None,
        params_override: params_arg,
        since_secs,
        own: discovery::PlayerId(own_id),
        transport: mode,
        room_tx,
        room_requests: request_rx,
        directory_tx,
    };
    tokio::spawn(discovery::run(run_config));
    let room = lobby_arg.unwrap_or_default();
    if args.create_lobby {
        tracing::info!("lobby {} created (cap {})", room, clicker::LOBBY_CAP);
    }
    let mut app = App::new();
    app.insert_resource(clicker::InstanceInfo {
        namespace: args.namespace,
        instance_tag: args.instance_tag,
        own_id: net_id::NetworkId(own_id),
    });
    app.insert_resource(clicker::ActiveLobby(room.clone()));
    app.insert_resource(clicker::GlobalCounter::default());
    app.insert_resource(clicker::PendingClicks::default());
    app.insert_resource(lobby::DirectoryFeed(std::sync::Mutex::new(Some(
        directory_rx,
    ))));
    app.insert_resource(lobby::RoomRequestTx(request_tx));
    app.insert_resource(lobby::RoomRx(std::sync::Mutex::new(Some(room_rx))));
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: format!("clicker-{} [{}]", args.instance_tag, room),
            resolution: (800, 450).into(),
            ..default()
        }),
        primary_cursor_options: Some(CursorOptions {
            visible: false,
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(P2PPlugin(Config::<clicker::CursorMsg>::new(
        net_id::NetworkId(own_id),
        cmd_tx,
        bevy_rx,
    )));
    app.insert_resource(roster::Lobby(room.clone()));
    app.insert_resource(lobby::SelectedRoom(if room.is_empty() {
        None
    } else {
        Some(room.clone())
    }));
    app.add_plugins(clicker::Plugin);
    app.add_plugins(lobby::Plugin);
    if !room.is_empty() {
        app.world_mut()
            .resource_mut::<NextState<lobby::AppState>>()
            .set(lobby::AppState::InRoom);
    }
    app.run();
}

// needed helper: maps the CLI transport flag onto the swarm transport mode
const fn transport_mode(arg: &TransportArg) -> p2p::TransportMode {
    match arg {
        TransportArg::Tcp => p2p::TransportMode::Tcp,
        TransportArg::Quic => p2p::TransportMode::Quic,
        TransportArg::Both => p2p::TransportMode::Both,
    }
}

async fn forward_events(
    mut raw_rx: tokio::sync::mpsc::UnboundedReceiver<p2p::Event<clicker::CursorMsg>>,
    bevy_tx: tokio::sync::mpsc::UnboundedSender<p2p::Event<clicker::CursorMsg>>,
    ready_tx: tokio::sync::watch::Sender<Option<(String, Vec<String>)>>,
    obs_tx: tokio::sync::watch::Sender<Option<Vec<String>>>,
    link_tx: tokio::sync::mpsc::UnboundedSender<(String, bool)>,
    lobby_tx: tokio::sync::mpsc::UnboundedSender<p2p::Event<clicker::CursorMsg>>,
) {
    while let Some(event) = raw_rx.recv().await {
        if let p2p::Event::Ready { peer_id, addrs } = &event {
            ready_tx.send_replace(Some((peer_id.clone(), addrs.clone())));
        }
        if let p2p::Event::ObservedAddr(addr) = &event {
            obs_tx.send_replace(Some(vec![addr.clone()]));
        }
        match &event {
            p2p::Event::PeerConnected(peer) => {
                tracing::info!("p2p connected peer={peer}");
                link_tx.send((peer.clone(), true)).ok();
                lobby_tx.send(p2p::Event::PeerConnected(peer.clone())).ok();
            }
            p2p::Event::PeerDisconnected(peer) => {
                tracing::info!("p2p disconnected peer={peer}");
                link_tx.send((peer.clone(), false)).ok();
            }
            p2p::Event::LobbyProviders { .. } | p2p::Event::Gossip { .. } => {
                lobby_tx.send(event.clone()).ok();
            }
            p2p::Event::DialFailed { peer_id, reason } => {
                tracing::warn!("p2p dial failed peer={peer_id} reason={reason}");
            }
            p2p::Event::RelayReserved { relay_peer_id } => {
                tracing::info!("p2p relay reserved relay={relay_peer_id}");
            }
            _ => {}
        }
        if bevy_tx.send(event).is_err() {
            break;
        }
    }
}
