use bevy::app::App;
use tokio::task::JoinHandle;

use crate::methods;

/// Like `TestGameApp`, but exercises the exact production node-startup sequence
/// (`p2p::load_or_create_keypair` -> `p2p::run` -> `roster::start_embedded_node` ->
/// `roster::connect_client_loop`, wired into a Bevy `App` with `p2p::Plugin` included) instead
/// of the hermetic `TestNode` harness. `TestGameApp` never exercises `p2p::Plugin` or the real
/// node-discovery path, so it can't catch bugs specific to those — this struct exists to close
/// that gap. Contract isolation still uses `unique_params()`, not the empty params production
/// uses, so tests don't write into the real shared roster contract.
pub struct ProductionGameApp {
    pub(crate) app: App,
    pub(crate) _p2p_task: JoinHandle<()>,
    pub(crate) _roster_task: JoinHandle<()>,
    pub(crate) _identity_dir: tempfile::TempDir,
    pub(crate) _node_dir: tempfile::TempDir,
    pub(crate) gateway: String,
}

#[rustfmt::skip]
impl ProductionGameApp {
    pub async fn new(wasm: &[u8], params: &[u8], player_index: u64) -> Self {
        methods::production_game_app::new(wasm, params, player_index).await
    }
    pub async fn new_local(wasm: &[u8], params: &[u8], player_index: u64, gateway: Option<String>) -> Self {
        methods::production_game_app::new_local(wasm, params, player_index, gateway).await
    }
    pub fn box_count(&mut self) -> usize { methods::production_game_app::box_count(self) }
    pub fn roster_len(&mut self) -> usize { methods::production_game_app::roster_len(self) }
    pub fn roster_ids(&self) -> Vec<freenet_libp2p_bevy_example_3_lib::boxes::PlayerId> { methods::production_game_app::roster_ids(self) }
    pub fn own_player_id(&self) -> freenet_libp2p_bevy_example_3_lib::boxes::PlayerId { methods::production_game_app::own_player_id(self) }
    pub fn box_spawns(&mut self) -> Vec<(freenet_libp2p_bevy_example_3_lib::boxes::PlayerId, bevy::math::Vec2, bool)> { methods::production_game_app::box_spawns(self) }
    pub fn remote_snapshots(&mut self) -> Vec<(freenet_libp2p_bevy_example_3_lib::boxes::PlayerId, bevy::math::Vec2, u64)> { methods::production_game_app::remote_snapshots(self) }
    pub fn state_hash(&self) -> u64 { methods::production_game_app::state_hash(self) }
    pub fn debug_snapshot(&self) -> String { methods::production_game_app::debug_snapshot(self) }
    pub fn own_box_position(&self) -> Option<bevy::math::Vec2> { methods::production_game_app::own_box_position(self) }
    pub fn freenet_gateway(&self) -> String { methods::production_game_app::freenet_gateway(self) }
    pub async fn wait_for_box_count(&mut self, expected: usize, timeout: std::time::Duration) -> Result<(), String> {
        methods::production_game_app::wait_for_box_count(self, expected, timeout).await
    }
    pub async fn wait_for_roster_len(&mut self, expected: usize, timeout: std::time::Duration) -> Result<(), String> {
        methods::production_game_app::wait_for_roster_len(self, expected, timeout).await
    }
    pub async fn wait_for_box_ids(&mut self, expected: &[freenet_libp2p_bevy_example_3_lib::boxes::PlayerId], timeout: std::time::Duration) -> Result<(), String> {
        methods::production_game_app::wait_for_box_ids(self, expected, timeout).await
    }
    pub async fn wait_for_roster_ids(&mut self, expected: &[freenet_libp2p_bevy_example_3_lib::boxes::PlayerId], timeout: std::time::Duration) -> Result<(), String> {
        methods::production_game_app::wait_for_roster_ids(self, expected, timeout).await
    }
    pub fn simulate_move(&mut self, direction: bevy::input::keyboard::KeyCode, frames: u32) {
        methods::production_game_app::simulate_move(self, direction, frames)
    }
    pub fn press_key(&mut self, key: bevy::input::keyboard::KeyCode) {
        methods::production_game_app::press_key(self, key)
    }
    pub fn release_key(&mut self, key: bevy::input::keyboard::KeyCode) {
        methods::production_game_app::release_key(self, key)
    }
    pub fn simulate_move_and_jump(&mut self, direction: bevy::input::keyboard::KeyCode, frames: u32) {
        methods::production_game_app::simulate_move_and_jump(self, direction, frames)
    }
    pub fn tick(&mut self) { self.app.update(); }
    pub fn sim_clock(&mut self) -> u64 { methods::production_game_app::sim_clock(self) }
}

impl Drop for ProductionGameApp {
    fn drop(&mut self) {
        self._p2p_task.abort();
        self._roster_task.abort();
    }
}
