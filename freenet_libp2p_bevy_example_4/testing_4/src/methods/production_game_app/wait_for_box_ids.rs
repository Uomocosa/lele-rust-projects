use std::time::{Duration, Instant};

use freenet_libp2p_bevy_example_4_lib::boxes;

use crate::structs;

pub async fn wait_for_box_ids(
    this: &mut structs::ProductionGameApp,
    expected: &[boxes::PlayerId],
    timeout: Duration,
) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    loop {
        this.app.update();
        let ids: Vec<boxes::PlayerId> =
            this.box_spawns().into_iter().map(|(id, _, _)| id).collect();
        if expected.iter().all(|id| ids.contains(id)) {
            return Ok(());
        }
        if Instant::now() > deadline {
            return Err(format!(
                "no box spawned for all {expected:?} before {timeout:?} timeout (have {ids:?})"
            ));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
// no test_usage necessary — needs a live embedded freenet node, exercised by tests/
