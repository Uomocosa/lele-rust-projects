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
        if tick_n.is_multiple_of(MOVE_EVERY_TICKS) {
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
