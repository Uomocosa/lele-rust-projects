use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::Serialize;
use testing_4::ProductionGameApp;

const DEFAULT_DURATION_SECS: u64 = 300;
const POLL_INTERVAL_SECS: u64 = 2;
// Mirrors JOIN_STAGGER_SECS in the mainnet probes: without a stagger, both machines can
// independently miss each other's first `Put` of the shared contract key and seed disjoint
// replicas that only reconcile via freenet's 5-minute InterestSync anti-entropy.
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
    own: String,
    observed: Vec<String>,
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
    std::env::var("CROSS_OS_LOG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(format!("cross-os-{machine}.log")))
}

// needed helper:
fn contract_params() -> Vec<u8> {
    match std::env::var("CROSS_OS_KEY") {
        Ok(key) => key.into_bytes(),
        Err(_) => testing_4::unique_params(),
    }
}

// needed helper:
fn window_secs() -> u64 {
    std::env::var("CROSS_OS_DURATION_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_DURATION_SECS)
}

/// The cross-OS roster-presence probe. Each machine (Linux + Windows runner) runs this same
/// `#[tokio::test]` against the same `CROSS_OS_KEY` contract on the public Freenet mainnet and
/// writes the roster it observes to a JSON-lines log. The workflow's `cross-os-verify` job
/// downloads both logs and asserts each side saw the other's player id — that is what makes it a
/// cross-OS test regardless of whether the machines share a LAN.
///
/// Player ids are `[u8; 32]`, logged as lowercase hex strings.
///
/// `#[ignore]`d — run per-machine from the workflow with
/// `cargo test -p cross_os_tests_4 --test peer_discovery -- --ignored`.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn peer_discovery() -> Result<(), Box<dyn std::error::Error>> {
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

    let wasm = testing_4::load_wasm();
    let params = contract_params();
    let mut app = ProductionGameApp::new(&wasm, &params, 0).await;
    let own = hex::encode(app.own_player_id());

    let mut file = File::create(&path)?;
    let start = Instant::now();
    let deadline = start + Duration::from_secs(window_secs());
    let mut last_observed: Vec<String> = Vec::new();

    loop {
        app.tick();
        let observed: Vec<String> = app.roster_ids().iter().map(hex::encode).collect();
        if observed != last_observed {
            let line = LogLine {
                machine: machine.clone(),
                own: own.clone(),
                observed: observed.clone(),
                t: start.elapsed().as_secs_f64(),
            };
            let mut text = serde_json::to_string(&line)?;
            text.push('\n');
            file.write_all(text.as_bytes())?;
            file.flush()?;
            last_observed = observed;
        }
        if Instant::now() >= deadline {
            break;
        }
        tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }

    drop(app);
    tracing::info!(target: "roster", machine = %machine, own = %own, final_observed = ?last_observed, "cross-os window complete");
    Ok(())
}
