use super::build;

use crate::structs;

pub async fn new(wasm: &[u8], params: &[u8], player_index: u64) -> structs::ProductionGameApp {
    build::build(wasm, params, player_index, false, None).await
}
// no test_usage necessary — needs a live embedded freenet node, exercised by tests/
