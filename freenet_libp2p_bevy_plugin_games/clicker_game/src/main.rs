use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use clap::Parser;
use clicker_game_lib::clicker;
use freenet_libp2p_bevy_plugin::{cli, p2p, plugin, roster};

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[tokio::main]
async fn main() {
    let cli = cli::Cli::parse();

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "warn,roster=info,p2p=info,clicker_game=info".into());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stdout)
        .init();

    let keypair = p2p::load_or_create_keypair(cli.identity_dir);
    let own_id = p2p::derive_network_id(&keypair);

    let (cmd_tx, cmd_rx) =
        tokio::sync::mpsc::unbounded_channel::<p2p::Command<clicker::ClickDelta>>();
    let (event_tx, mut event_rx) =
        tokio::sync::mpsc::unbounded_channel::<p2p::Event<clicker::ClickDelta>>();

    tokio::spawn(p2p::run(cmd_rx, event_tx, keypair));

    let (peer_id, addrs) = match event_rx.recv().await {
        Some(p2p::Event::Ready { peer_id, addrs }) => (peer_id, addrs),
        Some(p2p::Event::Error(e)) => {
            tracing::error!("p2p failed: {e}");
            return;
        }
        _ => return,
    };
    let own_entry = roster::PeerEntry {
        peer_id: peer_id.clone(),
        addrs: addrs
            .into_iter()
            .map(|addr| format!("{addr}/p2p/{peer_id}"))
            .collect(),
        updated_at: now_unix_secs(),
    };

    let contract_wasm = include_bytes!("../contract/roster_contract.wasm").to_vec();
    let (roster_tx, roster_rx) = tokio::sync::mpsc::unbounded_channel::<roster::Event>();

    tokio::spawn(roster::connect_and_run(
        roster::ConnectAndRunArgs {
            local: cli.freenet_local,
            gateway: cli.freenet_gateway,
            contract_wasm,
            params: cli
                .contract_params
                .map(String::into_bytes)
                .unwrap_or_default(),
            own_id,
            own_entry,
        },
        roster_tx,
    ));

    App::new()
        .add_plugins(DefaultPlugins.build().disable::<bevy::log::LogPlugin>())
        .add_plugins(plugin::Plugin(plugin::Config::new(
            own_id, cmd_tx, event_rx, roster_rx,
        )))
        .add_plugins(clicker::Plugin)
        .run();
}
