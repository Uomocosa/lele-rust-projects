use std::time::{Duration, Instant};

use crate::structs::production_game_app::ProductionGameApp;

pub async fn wait_for_roster_len(
    this: &mut ProductionGameApp,
    expected: usize,
    timeout: Duration,
) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    loop {
        this.app.update();
        if this.roster_len() >= expected {
            return Ok(());
        }
        if Instant::now() > deadline {
            return Err(format!(
                "roster did not reach {expected} entries before {timeout:?} timeout"
            ));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
// no test_usage necessary — needs a live embedded freenet node, exercised by tests/
