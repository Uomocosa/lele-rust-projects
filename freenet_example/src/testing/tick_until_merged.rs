use std::time::{Duration, Instant};

use crate::clicker_client;

/// Tick until a foreign slot is observed or the deadline elapses.
pub async fn tick_until_merged(
    client: &mut clicker_client::ClickerClient,
    deadline: Duration,
) -> (u64, Vec<u64>, u64) {
    let start = Instant::now();
    let mut ticks = 0u64;
    loop {
        match client.tick().await {
            Ok(count) => {
                ticks = ticks.wrapping_add(1);
                println!(
                    "tick tag={} count={count} owns={} ticks={ticks}",
                    client.tag,
                    client.own()
                );
            }
            Err(e) => eprintln!("tick error: {e}"),
        }
        client.note_foreign_slots();
        if let Err(e) = client.bridge_tick().await {
            eprintln!("bridge error: {e}");
        }
        let merged = !client.foreign_tags().is_empty() && ticks >= 30;
        if merged || start.elapsed() >= deadline {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    let foreign_tags: Vec<u64> = client.foreign_tags();
    let count = client.count();
    let elapsed = start.elapsed().as_secs();
    let _ = elapsed;
    (ticks, foreign_tags, count)
}

// no test_usage necessary
