use bevy::app::App;
use tokio::task::JoinHandle;

use crate::methods::test_game_app as tga_method;

pub struct TestGameApp {
    pub(crate) app: App,
    pub(crate) _roster_task: JoinHandle<()>,
}

#[rustfmt::skip]
impl TestGameApp {
    pub fn new(ws_port: u16, wasm: &[u8], params: &[u8], own_id: freenet_libp2p_bevy_example_1_lib::boxes::PlayerId, own_entry: freenet_libp2p_bevy_example_1_lib::roster::PeerEntry) -> Self {
        tga_method::new(ws_port, wasm, params, own_id, own_entry)
    }
    pub fn box_count(&mut self) -> usize { tga_method::box_count(self) }
    pub fn roster_len(&mut self) -> usize { tga_method::roster_len(self) }
    pub fn box_spawns(&mut self) -> Vec<(freenet_libp2p_bevy_example_1_lib::boxes::PlayerId, bevy::math::Vec2, bool)> { tga_method::box_spawns(self) }
    pub async fn wait_for_box_count(&mut self, expected: usize, timeout: std::time::Duration) -> Result<(), String> {
        tga_method::wait_for_box_count(self, expected, timeout).await
    }
    pub async fn wait_for_roster_len(&mut self, expected: usize, timeout: std::time::Duration) -> Result<(), String> {
        tga_method::wait_for_roster_len(self, expected, timeout).await
    }
}

impl Drop for TestGameApp {
    fn drop(&mut self) {
        self._roster_task.abort();
    }
}
