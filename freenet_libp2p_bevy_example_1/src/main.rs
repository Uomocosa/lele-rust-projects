use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use bevy_freenet::{boxes, cli, p2p, roster};

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stdout)
        .init();

    let own_id = boxes::PlayerId(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0),
    );
    let p2p_port = cli::parse_p2p_port().unwrap_or(0);

    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(p2p::run(cmd_rx, event_tx));

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
