use std::time::{Duration, Instant};

use crate::structs::test_game_app::TestGameApp;

pub async fn wait_for_box_count(
    this: &mut TestGameApp,
    expected: usize,
    timeout: Duration,
) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    loop {
        this.app.update();
        if this.box_count() >= expected {
            return Ok(());
        }
        if Instant::now() > deadline {
            return Err(format!(
                "box count did not reach {expected} before {timeout:?} timeout"
            ));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
// no test_usage necessary — needs a live embedded freenet node, exercised by tests/
