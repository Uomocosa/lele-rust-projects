use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use freenet_libp2p_bevy_example_3_lib::engine;
use serde::Serialize;

/// Length of the fixed input trace both machines replay through the engine.
const TRACE_TICKS: u64 = 120;

#[derive(Serialize)]
struct LogLine {
    machine: String,
    final_state_hash: u64,
    t: f64,
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
    std::env::var("CROSS_OS_LOG_DETERMINISM")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(format!("cross-os-determinism-{machine}.log")))
}

// needed helper:
/// The fixed input trace every machine replays: deterministic constants only, so Linux and
/// Windows feed byte-identical inputs into the engine (POLISH §3 cross-OS determinism).
fn canonical_trace() -> Vec<(u64, Vec<(engine::PlayerId, engine::Action)>)> {
    (0..TRACE_TICKS)
        .map(|tick| {
            let actions = vec![
                (
                    [1u8; 32],
                    engine::Action {
                        direction: engine::Direction::Right,
                        jump: tick == 20,
                    },
                ),
                (
                    [2u8; 32],
                    engine::Action {
                        direction: if tick % 3 == 0 {
                            engine::Direction::Left
                        } else if tick % 2 == 0 {
                            engine::Direction::Right
                        } else {
                            engine::Direction::Center
                        },
                        jump: false,
                    },
                ),
            ];
            (tick, actions)
        })
        .collect()
}

fn now_unix_secs() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// The cross-OS engine determinism gate (POLISH §3). Each machine runs the **same fixed input
/// trace** through the plain authoritative engine and writes the resulting **final state hash**
/// to its own JSON-lines log. The workflow's `cross-os-verify` job downloads both logs and
/// asserts the hashes are identical — proving avian's `enhanced-determinism` holds across OS.
///
/// Pure-local (no network, no window): can run at any time on either machine.
///
/// `#[ignore]`d — run per-machine from the workflow with
/// `cargo test -p cross_os_tests_3 --test engine_determinism -- --ignored`.
#[test]
#[ignore]
fn engine_determinism_gate() -> Result<(), Box<dyn std::error::Error>> {
    let machine = machine_label();
    let path = log_path(&machine);
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }

    let trace = canonical_trace();
    let hashes = engine::run_trace(&trace);
    let final_state_hash = *hashes.last().ok_or("engine produced no state hashes")?;

    tracing::info!(
        target: "engine",
        machine = %machine,
        final_state_hash,
        "cross-os determinism gate complete"
    );

    let line = LogLine {
        machine,
        final_state_hash,
        t: now_unix_secs(),
    };
    let mut text = serde_json::to_string(&line)?;
    text.push('\n');
    File::create(&path)?.write_all(text.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{TRACE_TICKS, canonical_trace};
    use freenet_libp2p_bevy_example_3_lib::engine;

    #[test]
    fn test_usage() {
        let trace = canonical_trace();
        assert_eq!(trace.len() as u64, TRACE_TICKS);
        assert!(
            trace
                .iter()
                .all(|(tick, actions)| !actions.is_empty() && *tick < TRACE_TICKS)
        );

        let first = engine::run_trace(&trace);
        let second = engine::run_trace(&trace);
        assert_eq!(first.len(), second.len());
        assert_eq!(
            first.last(),
            second.last(),
            "the same in-process trace must hash identically"
        );
    }
}
