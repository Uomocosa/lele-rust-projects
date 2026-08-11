use super::build;

use crate::structs::production_game_app::ProductionGameApp;

pub async fn new_local(
    wasm: &[u8],
    params: &[u8],
    player_index: u64,
    gateway: Option<String>,
) -> ProductionGameApp {
    build::build(wasm, params, player_index, true, gateway).await
}
// no test_usage necessary — needs a live embedded freenet node, exercised by the local e2e test
