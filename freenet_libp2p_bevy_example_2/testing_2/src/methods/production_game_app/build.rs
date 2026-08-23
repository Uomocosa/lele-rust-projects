use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use freenet_libp2p_bevy_example_2_lib::{boxes, p2p, roster};

use crate::structs;

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Shared production-startup sequence: identity load -> p2p swarm -> embedded node ->
/// roster join -> Bevy app wiring. Substitutes `params` for production's hardcoded empty slice so
/// tests don't write into the real shared roster contract, and an isolated identity dir per
/// instance so multiple apps in one test process don't collide the way same-`$HOME` `cargo run`
/// instances used to. `local`/`gateway` select the embedded node's join mode (see
/// `roster::start_embedded_node`).
pub(crate) async fn build(
    wasm: &[u8],
    params: &[u8],
    player_index: u64,
    local: bool,
    gateway: Option<String>,
) -> structs::ProductionGameApp {
    let identity_dir = tempfile::tempdir().expect("create identity temp dir");
    let identity_path = identity_dir.path().join(format!("player-{player_index}"));

    let keypair = p2p::load_or_create_keypair(Some(identity_path));
    let own_id = p2p::derive_player_id(&keypair);

    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let p2p_task = tokio::spawn(p2p::run(cmd_rx, event_tx, keypair.clone()));

    let (peer_id, addrs) = match event_rx.recv().await {
        Some(p2p::Event::Ready { peer_id, addrs }) => (peer_id, addrs),
        Some(p2p::Event::Error(e)) => panic!("p2p failed to start: {e}"),
        other => panic!("expected p2p::Event::Ready, got {other:?}"),
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

    let node = loop {
        match roster::start_embedded_node(local, gateway.clone()).await {
            Ok(node) => break node,
            Err(e) => {
                tracing::error!(target: "roster", error = %e, "embedded node startup failed, retrying");
                tokio::time::sleep(Duration::from_secs(roster::NODE_START_RETRY_BACKOFF_SECS))
                    .await;
            }
        }
    };
    let roster::NodeInfo {
        host,
        ws_port,
        public_port,
        public_key_hex,
        node_dir,
    } = node;
    let gateway = format!("127.0.0.1:{public_port},{public_key_hex}");

    let (roster_tx, roster_rx) = tokio::sync::mpsc::unbounded_channel();
    let wasm = wasm.to_vec();
    let params = params.to_vec();
    let not_found_grace = if local {
        std::time::Duration::ZERO
    } else {
        std::time::Duration::from_secs(roster::SETUP_CONTRACT_GRACE_SECS)
    };
    let roster_task = tokio::spawn(async move {
        roster::connect_client_loop(
            roster::ConnectClientArgs {
                host: &host,
                port: ws_port,
                contract_wasm: &wasm,
                params: &params,
                own_keypair: keypair.clone(),
                own_id,
                own_entry,
                not_found_grace,
            },
            roster_tx,
        )
        .await;
    });

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::transform::TransformPlugin);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_secs_f32(1.0 / 60.0),
    ));
    app.add_plugins(boxes::Plugin(boxes::Config::new(own_id)));
    app.add_plugins(roster::Plugin(roster::Config::new(roster_rx)));
    app.add_plugins(p2p::Plugin(p2p::Config::new(cmd_tx, event_rx)));
    app.finish();

    structs::ProductionGameApp {
        app,
        _p2p_task: p2p_task,
        _roster_task: roster_task,
        _identity_dir: identity_dir,
        _node_dir: node_dir,
        gateway,
    }
}
// no test_usage necessary — needs a live embedded freenet node, exercised by testing/
