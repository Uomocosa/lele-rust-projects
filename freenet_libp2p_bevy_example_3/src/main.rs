use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use clap::Parser;
use freenet_libp2p_bevy_example_3_lib::{boxes, cli, p2p, roster};
use serde::{Deserialize, Serialize};

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct Params {
    namespace: [u8; 32],
    max_members: u16,
}

#[tokio::main]
async fn main() {
    let cli = cli::Cli::parse();

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "warn,roster=info,p2p=info".into());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stdout)
        .init();

    let keypair = p2p::load_or_create_keypair(cli.identity_dir);
    let own_id = p2p::derive_player_id(&keypair);

    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(p2p::run(cmd_rx, event_tx, keypair.clone()));

    let (peer_id, addrs) = match event_rx.recv().await {
        Some(p2p::Event::Ready { peer_id, addrs }) => (peer_id, addrs),
        Some(p2p::Event::Error(e)) => {
            tracing::error!("p2p failed: {e}");
            return;
        }
        _ => return,
    };
    let addrs: Vec<String> = addrs
        .into_iter()
        .map(|addr| format!("{addr}/p2p/{peer_id}"))
        .collect();
    let seq = now_unix_secs();
    let own_entry = roster::PeerEntry {
        peer_id: peer_id.clone(),
        addrs: addrs.clone(),
        seq,
        signature: roster::sign_entry(&keypair, &peer_id, &addrs, seq),
    };

    let namespace = {
        let mut ns = [0u8; 32];
        if let Some(s) = cli.contract_params.as_deref() {
            let bytes = s.as_bytes();
            let n = bytes.len().min(32);
            ns[..n].copy_from_slice(&bytes[..n]);
        }
        ns
    };
    let params = bincode::serialize(&Params {
        namespace,
        max_members: 64,
    })
    .unwrap_or_default();

    let contract_wasm = include_bytes!("../contract/membership_contract.wasm").to_vec();
    let (roster_tx, roster_rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(roster::connect_and_run(
        roster::ConnectAndRunArgs {
            local: cli.freenet_local,
            gateway: cli.freenet_gateway,
            contract_wasm,
            params,
            own_keypair: keypair,
            own_id,
            own_entry,
        },
        roster_tx,
    ));

    App::new()
        .add_plugins(DefaultPlugins.build().disable::<bevy::log::LogPlugin>())
        .add_plugins(boxes::Plugin(boxes::Config::new(own_id)))
        .add_plugins(roster::Plugin(roster::Config::new(roster_rx)))
        .add_plugins(p2p::Plugin(p2p::Config::new(cmd_tx, event_rx)))
        .run();
}
