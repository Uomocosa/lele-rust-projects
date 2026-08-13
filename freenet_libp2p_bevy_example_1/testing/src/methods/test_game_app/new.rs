use bevy::prelude::*;
use freenet_libp2p_bevy_example_1_lib::{boxes, roster};

use crate::structs::test_game_app::TestGameApp;

pub fn new(
    ws_port: u16,
    wasm: &[u8],
    params: &[u8],
    own_id: boxes::PlayerId,
    own_entry: roster::PeerEntry,
) -> TestGameApp {
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
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
                own_id,
                own_entry,
                not_found_grace: std::time::Duration::ZERO,
            },
            event_tx,
        )
        .await;
    });

    TestGameApp {
        app,
        _roster_task: roster_task,
    }
}
// no test_usage necessary — needs a live embedded freenet node, exercised by tests/
