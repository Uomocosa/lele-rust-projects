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
    let _runner = p2p::spawn_runner(cmd_rx, raw_tx);
    tokio::spawn(forward_events(
        raw_rx, bevy_tx, ready_tx, obs_tx, link_tx, lobby_tx,
    ));
    let lobby_arg = args.lobby.clone();
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
        lobby: lobby_arg.clone(),
        params_override: params_arg,
        since_secs,
        own: discovery::PlayerId(own_id),
        room_tx,
    };
    tokio::spawn(discovery::run(run_config));
    let room = wait_room(room_rx, lobby_arg.clone()).await;
    if args.create_lobby {
        tracing::info!("lobby {} created (cap {})", room, clicker::LOBBY_CAP);
    }
    App::new()
        .insert_resource(clicker::InstanceInfo {
            namespace: args.namespace,
            instance_tag: args.instance_tag,
            own_id: net_id::NetworkId(own_id),
        })
        .insert_resource(clicker::ActiveLobby(room.clone()))
        .insert_resource(clicker::GlobalCounter::default())
        .insert_resource(clicker::PendingClicks::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
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
        }))
        .add_plugins(P2PPlugin(Config::<clicker::CursorMsg>::new(
            net_id::NetworkId(own_id),
            cmd_tx,
            bevy_rx,
        )))
        .insert_resource(roster::Lobby(room))
        .add_plugins(clicker::Plugin)
        .run();
}

// needed helper: waits for discovery to resolve the room (creator lobby is immediate)
async fn wait_room(
    mut room_rx: tokio::sync::watch::Receiver<Option<String>>,
    lobby_arg: Option<String>,
) -> String {
    if let Some(room) = lobby_arg {
        return room;
    }
    let deadline = std::time::Duration::from_secs(320);
    if tokio::time::timeout(deadline, room_rx.wait_for(Option::is_some))
        .await
        .is_ok()
    {
        return room_rx.borrow().clone().unwrap_or_default();
    }
    String::new()
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
