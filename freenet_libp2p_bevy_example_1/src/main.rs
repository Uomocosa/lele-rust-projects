use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use freenet_libp2p_bevy_example_1_lib::{boxes, cli, p2p, roster};

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[tokio::main]
async fn main() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "warn,roster=info,p2p=info".into());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stdout)
        .init();

    let keypair = p2p::load_or_create_keypair(cli::parse_identity_dir());
    let own_id = p2p::derive_player_id(&keypair);
    let p2p_port = cli::parse_p2p_port().unwrap_or(0);

    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();

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
    let (roster_tx, roster_rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(roster::connect_and_run(
        p2p_port,
        cli::parse_freenet_local(),
        cli::parse_freenet_gateway(),
        contract_wasm,
        own_id,
        own_entry,
        roster_tx,
    ));

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(boxes::Plugin(boxes::Config::new(own_id)))
        .add_plugins(roster::Plugin(roster::Config::new(roster_rx)))
        .add_plugins(p2p::Plugin(p2p::Config::new(cmd_tx, event_rx)))
        .run();
}
