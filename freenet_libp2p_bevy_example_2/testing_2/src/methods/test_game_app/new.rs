use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use freenet_libp2p_bevy_example_2_lib::{boxes, p2p, roster};
use libp2p::identity::Keypair;

use crate::structs;

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn new(
    ws_port: u16,
    wasm: &[u8],
    params: &[u8],
    keypair: Keypair,
    peer_id: &str,
) -> structs::TestGameApp {
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
    let own_id = p2p::derive_player_id(&keypair);
    let seq = now_unix_secs();
    let own_entry = roster::PeerEntry {
        peer_id: peer_id.to_string(),
        addrs: vec![],
        seq,
        signature: roster::sign_entry(&keypair, peer_id, &[], seq),
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::transform::TransformPlugin);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.add_plugins(boxes::Plugin(boxes::Config::new(own_id)));
    app.add_plugins(roster::Plugin(roster::Config::new(event_rx)));
    app.finish();

    let wasm = wasm.to_vec();
    let params = params.to_vec();
    let roster_task = tokio::spawn(async move {
        roster::connect_client_loop(
            roster::ConnectClientArgs {
                host: "127.0.0.1",
                port: ws_port,
                contract_wasm: &wasm,
                params: &params,
                own_keypair: keypair,
                own_id,
                own_entry,
                not_found_grace: std::time::Duration::ZERO,
            },
            event_tx,
        )
        .await;
    });

    structs::TestGameApp {
        app,
        _roster_task: roster_task,
    }
}
// no test_usage necessary — needs a live embedded freenet node, exercised by tests/
