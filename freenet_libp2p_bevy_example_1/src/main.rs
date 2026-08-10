use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use bevy_freenet::{boxes, roster};

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
    let own_entry = roster::PeerEntry {
        peer_id: format!("player-{}", *own_id),
        addrs: vec![],
        updated_at: now_unix_secs(),
    };

    let contract_wasm = include_bytes!("../contract/roster_contract.wasm").to_vec();
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(roster::connect_and_run(
        0,
        contract_wasm,
        own_id,
        own_entry,
        event_tx,
    ));

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(boxes::Plugin)
        .add_plugins(roster::Plugin(roster::Config::new(event_rx)))
        .run();
}
