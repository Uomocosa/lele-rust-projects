use std::time::{Duration, Instant};

use freenet_libp2p_bevy_example_1_lib::boxes;
use freenet_libp2p_bevy_example_1_lib::roster;

use crate::structs::production_game_app::ProductionGameApp;

pub async fn wait_for_roster_ids(
    this: &mut ProductionGameApp,
    expected: &[boxes::PlayerId],
    timeout: Duration,
) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    loop {
        this.app.update();
        let ids: Vec<boxes::PlayerId> = this
            .app
            .world()
            .resource::<roster::Roster>()
            .keys()
            .cloned()
            .collect();
        if expected.iter().all(|id| ids.contains(id)) {
            return Ok(());
        }
        if Instant::now() > deadline {
            return Err(format!(
                "roster did not contain all {expected:?} before {timeout:?} timeout (have {ids:?})"
            ));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
// no test_usage necessary — needs a live embedded freenet node, exercised by tests/
