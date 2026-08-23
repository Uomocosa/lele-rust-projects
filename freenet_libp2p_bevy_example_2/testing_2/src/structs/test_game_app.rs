use bevy::app::App;
use tokio::task::JoinHandle;

use crate::methods;

pub struct TestGameApp {
    pub(crate) app: App,
    pub(crate) _roster_task: JoinHandle<()>,
}

#[rustfmt::skip]
impl TestGameApp {
    pub fn new(ws_port: u16, wasm: &[u8], params: &[u8], keypair: libp2p::identity::Keypair, peer_id: &str) -> Self {
        methods::test_game_app::new(ws_port, wasm, params, keypair, peer_id)
    }
    pub fn box_count(&mut self) -> usize { methods::test_game_app::box_count(self) }
    pub fn roster_len(&mut self) -> usize { methods::test_game_app::roster_len(self) }
    pub fn box_spawns(&mut self) -> Vec<(freenet_libp2p_bevy_example_2_lib::boxes::PlayerId, bevy::math::Vec2, bool)> { methods::test_game_app::box_spawns(self) }
    pub async fn wait_for_box_count(&mut self, expected: usize, timeout: std::time::Duration) -> Result<(), String> {
        methods::test_game_app::wait_for_box_count(self, expected, timeout).await
    }
    pub async fn wait_for_roster_len(&mut self, expected: usize, timeout: std::time::Duration) -> Result<(), String> {
        methods::test_game_app::wait_for_roster_len(self, expected, timeout).await
    }
}

impl Drop for TestGameApp {
    fn drop(&mut self) {
        self._roster_task.abort();
    }
}
