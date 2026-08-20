use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use bevy::input::keyboard::KeyCode;
use bevy::math::Vec2;
use serde::Serialize;
use testing::ProductionGameApp;

const DEFAULT_DURATION_SECS: u64 = 300;
const MOVE_EVERY_TICKS: u64 = 30;
const MOVE_FRAMES: u32 = 20;
const SAMPLE_EVERY_TICKS: u64 = 5;
// Two boxes walking straight at each other pin against each other's collider and barely
// move (Avian2d resolves the overlap by opposing each side's velocity), which can starve
// cross-os-verify's "remote box moved" check. When boxes are this close, jump-and-move
// away instead of continuing the normal alternating walk.
const CLOSE_THRESHOLD: f32 = 60.0;
// ManualDuration(1.0/60.0) in build.rs makes each app.update() advance sim time by
// exactly 1/60s, so 60 frames is a deterministic ~1s hold.
const ESCAPE_FRAMES: u32 = 60;
// Mirrors JOIN_STAGGER_SECS in e2e_tests/e2e_three_node_production_sync.rs: without a
// stagger, both machines can independently miss each other's first `Put` of the shared
// contract key and seed disjoint replicas (see OBJECTIVE.md's InterestSync note).
const DEFAULT_JOIN_STAGGER_SECS: u64 = 45;

// needed helper:
fn join_stagger_secs() -> u64 {
    std::env::var("CROSS_OS_JOIN_STAGGER_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_JOIN_STAGGER_SECS)
}

#[derive(Serialize)]
struct LogLine {
    machine: String,
    own: u64,
    t: f64,
    local_x: f32,
    local_y: f32,
    remote_x: Option<f32>,
    remote_y: Option<f32>,
    remote_sent_at_ms: Option<u64>,
    lag_ms: Option<u64>,
}

// needed helper:
fn machine_label() -> String {
    std::env::var("CROSS_OS_MACHINE").unwrap_or_else(|_| {
        if cfg!(target_os = "windows") {
            "windows".to_string()
        } else {
            "linux".to_string()
        }
    })
}

// needed helper:
fn log_path(machine: &str) -> PathBuf {
    std::env::var("CROSS_OS_LOG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(format!("cross-os-movement-{machine}.log")))
}

// needed helper:
fn contract_params() -> Vec<u8> {
    match std::env::var("CROSS_OS_KEY") {
        Ok(key) => key.into_bytes(),
        Err(_) => testing::unique_params(),
    }
}

// needed helper:
fn window_secs() -> u64 {
    std::env::var("CROSS_OS_DURATION_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_DURATION_SECS)
}

// needed helper:
/// Direction that increases separation from `remote_x`, or `None` if already clear.
fn escape_direction(local_x: f32, remote_x: f32) -> Option<KeyCode> {
    if (local_x - remote_x).abs() >= CLOSE_THRESHOLD {
        return None;
    }
    Some(if local_x < remote_x {
        KeyCode::KeyA
    } else {
        KeyCode::KeyD
    })
}

/// If the local box is close enough to the remote box that they'd collide head-on,
/// jump-and-move away from it. Returns whether an escape maneuver was performed.
fn move_to_empty_space(app: &mut ProductionGameApp, own: u64) -> bool {
    let spawns = app.box_spawns();
    let local_x = spawns
        .iter()
        .find(|(id, _, is_local)| *is_local || **id == own)
        .map(|(_, pos, _)| pos.x);
    let remote_x = spawns
        .iter()
        .find(|(id, _, is_local)| !*is_local && **id != own)
        .map(|(_, pos, _)| pos.x);

    match (local_x, remote_x) {
        (Some(local_x), Some(remote_x)) => match escape_direction(local_x, remote_x) {
            Some(direction) => {
                app.simulate_move_and_jump(direction, ESCAPE_FRAMES);
                true
            }
            None => false,
        },
        _ => false,
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// The cross-OS movement-sync probe. Both machines run this identical `#[tokio::test]`: each moves
/// its own box concurrently (no "one then the other"), and each samples **both** boxes' positions
/// every few ticks, writing the trace plus a **per-step lag** — `lag = local_now_ms -
/// remote_sent_at_ms`, i.e. how stale the observer's view of the remote box is, measured entirely
/// on the observer's own clock (no cross-machine clock arithmetic).
///
/// `#[ignore]`d — run per-machine from the workflow with
/// `cargo test --manifest-path cross_os_tests/Cargo.toml --test movement_sync -- --ignored`.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn movement_sync() -> Result<(), Box<dyn std::error::Error>> {
    let machine = machine_label();
    let path = log_path(&machine);
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }

    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn,roster=info,p2p=info".into()),
        )
        .try_init();

    if machine == "windows" {
        tokio::time::sleep(Duration::from_secs(join_stagger_secs())).await;
    }

    let wasm = testing::load_wasm();
    let params = contract_params();
    let mut app = ProductionGameApp::new(&wasm, &params, 0).await;
    let own = *app.own_player_id();

    let mut file = File::create(&path)?;
    let start = Instant::now();
    let deadline = start + Duration::from_secs(window_secs());
    let mut tick_n: u64 = 0;
    let mut last_remote_sent_at_ms: Option<u64> = None;

    while Instant::now() < deadline {
        if tick_n.is_multiple_of(MOVE_EVERY_TICKS) && !move_to_empty_space(&mut app, own) {
            let direction = if (tick_n / MOVE_EVERY_TICKS).is_multiple_of(2) {
                KeyCode::KeyD
            } else {
                KeyCode::KeyA
            };
            app.simulate_move(direction, MOVE_FRAMES);
        }
        app.tick();

        if tick_n.is_multiple_of(SAMPLE_EVERY_TICKS) {
            let now_local = now_ms();
            let t = start.elapsed().as_secs_f64();

            let spawns = app.box_spawns();
            let mut local_pos: Option<Vec2> = None;
            let mut remote_pos: Option<(u64, Vec2)> = None;
            for (id, pos, is_local) in &spawns {
                if *is_local || **id == own {
                    local_pos = Some(*pos);
                } else {
                    remote_pos = Some((**id, *pos));
                }
            }

            let snaps: BTreeMap<u64, u64> = app
                .remote_snapshots()
                .iter()
                .map(|(id, _, sent_at_ms)| (**id, *sent_at_ms))
                .collect();
            if let Some((remote_id, _)) = remote_pos
                && let Some(&sent) = snaps.get(&remote_id)
            {
                last_remote_sent_at_ms = Some(sent);
            }

            let line = LogLine {
                machine: machine.clone(),
                own,
                t,
                local_x: local_pos.map(|p| p.x).unwrap_or(f32::NAN),
                local_y: local_pos.map(|p| p.y).unwrap_or(f32::NAN),
                remote_x: remote_pos.map(|(_, p)| p.x),
                remote_y: remote_pos.map(|(_, p)| p.y),
                remote_sent_at_ms: last_remote_sent_at_ms,
                lag_ms: last_remote_sent_at_ms.map(|sent| now_local.saturating_sub(sent)),
            };
            let mut text = serde_json::to_string(&line)?;
            text.push('\n');
            file.write_all(text.as_bytes())?;
            file.flush()?;
        }

        tick_n += 1;
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    drop(app);
    tracing::info!(
        target: "roster",
        machine = %machine,
        own = own,
        last_remote_sent_at_ms = ?last_remote_sent_at_ms,
        "movement-sync window complete"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CLOSE_THRESHOLD, escape_direction};
    use bevy::input::keyboard::KeyCode;

    #[test]
    fn test_usage() {
        assert_eq!(escape_direction(0.0, CLOSE_THRESHOLD * 2.0), None);
        assert_eq!(escape_direction(0.0, 10.0), Some(KeyCode::KeyA));
        assert_eq!(escape_direction(10.0, 0.0), Some(KeyCode::KeyD));
    }
}
