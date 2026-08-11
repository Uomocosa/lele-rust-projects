use std::time::{SystemTime, UNIX_EPOCH};

use bevy::input::keyboard::KeyCode;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy_freenet::{boxes, p2p, roster};

use crate::structs::production_game_app::ProductionGameApp;

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Replicates `main.rs`'s exact startup sequence (identity load -> p2p swarm -> embedded node
/// -> roster join -> Bevy app wiring), substituting `params` for production's hardcoded empty
/// slice so tests don't write into the real shared roster contract, and an isolated identity
/// dir per instance so multiple `ProductionGameApp`s in one test process don't collide the way
/// same-`$HOME` `cargo run` instances used to.
pub async fn new(wasm: &[u8], params: &[u8], player_index: u64) -> ProductionGameApp {
    let identity_dir = tempfile::tempdir().expect("create identity temp dir");
    let identity_path = identity_dir.path().join(format!("player-{player_index}"));

    let keypair = p2p::load_or_create_keypair(Some(identity_path));
    let own_id = p2p::derive_player_id(&keypair);

    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let p2p_task = tokio::spawn(p2p::run(cmd_rx, event_tx, keypair));

    let (peer_id, addrs) = match event_rx.recv().await {
        Some(p2p::Event::Ready { peer_id, addrs }) => (peer_id, addrs),
        Some(p2p::Event::Error(e)) => panic!("p2p failed to start: {e}"),
        other => panic!("expected p2p::Event::Ready, got {other:?}"),
    };
    let own_entry = roster::PeerEntry {
        peer_id: peer_id.clone(),
        addrs: addrs
            .into_iter()
            .map(|addr| format!("{addr}/p2p/{peer_id}"))
            .collect(),
        updated_at: now_unix_secs(),
    };

    let (host, port, node_dir) = roster::start_embedded_node(0)
        .await
        .expect("start embedded freenet node");

    let (roster_tx, roster_rx) = tokio::sync::mpsc::unbounded_channel();
    let wasm = wasm.to_vec();
    let params = params.to_vec();
    let roster_task = tokio::spawn(async move {
        roster::connect_client_loop(&host, port, &wasm, &params, own_id, own_entry, roster_tx)
            .await;
    });

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::transform::TransformPlugin);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.add_plugins(boxes::Plugin(boxes::Config::new(own_id)));
    app.add_plugins(roster::Plugin(roster::Config::new(roster_rx)));
    app.add_plugins(p2p::Plugin(p2p::Config::new(cmd_tx, event_rx)));
    app.finish();

    ProductionGameApp {
        app,
        _p2p_task: p2p_task,
        _roster_task: roster_task,
        _identity_dir: identity_dir,
        _node_dir: node_dir,
    }
}
// no test_usage necessary — needs live internet access to the Freenet mainnet, exercised by
// the e2e test
