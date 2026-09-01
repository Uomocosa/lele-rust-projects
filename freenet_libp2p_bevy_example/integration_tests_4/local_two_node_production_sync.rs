use std::process::{Command, Stdio};
use std::time::Duration;

use bevy::input::keyboard::KeyCode;
use serde::Serialize;

const STEP_DELAY_MS: u64 = 5;

/// Deterministic-lockstep convergence across **two real OS processes**.
///
/// This is the faithful cross-process model: an in-process host joins the Freenet mainnet and a
/// real game binary subprocess joins the *same* contract. Each process runs its own physics on its
/// own threads (so no shared-process nondeterminism). Both peers drive the identical per-tick
/// ordered input set over libp2p and must end on the same engine state hash — asserted here as the
/// host reporting **zero state-hash divergences** against the guest on its final tick.
///
/// This supersedes the older single-process harness, which ran two engine `App`s in one process and
/// was inherently nondeterministic (Avian's integrator draws from a process-wide bevy `ComputeTaskPool`,
/// so two co-existing engines interleave and cannot be bit-identical no matter how correctly the lockstep
/// is driven). Production and mainnet are separate processes, so this subprocess test is the representative
/// gate.
/// This is a live-mainnet, multi-process test, so it is slow (typically 2–4 minutes) and depends on
/// the real network — it is `#[ignore]`d by default so it never gates a routine `cargo test`; run it
/// explicitly (`cargo test -p integration_tests_4 --test local_two_node_production_sync -- --ignored`)
/// or rely on the per-iteration `mainnet-automation-4` for the regular cross-process gate.
///
/// Its previous flakiness was root-caused and fixed: (1) `setup_contract` stranded on a timed-out
/// `Get` of a brand-new key instead of seeding via `Put`; (2) the 2-node live-join snapshot
/// cross-check required `>=2` peers, so a late joiner never re-baselined; (3) both peers
/// mutually re-adopted snapshots, rewinding each other. With those fixed it passes reliably
/// (3 consecutive runs at ~2:20 avg).
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn two_node_production_sync() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn,roster=trace,p2p=info".into()),
        )
        .with_writer(std::io::stdout)
        .try_init();

    // Locate the game binary next to this test executable: target/{profile}/deps/<this> ->
    // target/{profile}/freenet-libp2p-bevy-example. `CARGO_BIN_EXE_*` is only set within the
    // bin's own package, so resolve via the current test binary's directory instead.
    let exe = std::env::current_exe()?;
    let target = exe
        .parent()
        .and_then(|deps| deps.parent())
        .ok_or("cannot locate target dir from test executable")?;
    let bin = target.join("freenet-libp2p-bevy-example");
    if !bin.exists() {
        return Err(format!("game binary not found at {}", bin.display()).into());
    }
    let wasm = testing_4::load_wasm();

    // A shared contract namespace expressed as an ASCII string. The game binary derives its
    // `Params` from `--contract-params` (namespace = first bytes of the string, max_members = 64),
    // so we build the host's identical  `Params` and hand the same string to the guest.
    let ns = format!(
        "twoprocess-{}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
        std::process::id()
    );
    let mut namespace = [0u8; 32];
    namespace[..ns.len().min(32)].copy_from_slice(&ns.as_bytes()[..ns.len().min(32)]);
    let params = bincode::serialize(&Params {
        namespace,
        max_members: 64,
    })?;

    let guest_identity = tempfile::tempdir()?;
    let guest_log_path = guest_identity.path().join("guest.log");
    let guest_log = std::fs::File::create(&guest_log_path)?;
    let guest_err = guest_log.try_clone()?;
    let mut guest = Command::new(&bin)
        .env("RUST_LOG", "warn,roster=trace,p2p=info")
        .arg("--identity-dir")
        .arg(guest_identity.path().join("player-guest"))
        .arg("--contract-params")
        .arg(&ns)
        .stdout(Stdio::from(guest_log))
        .stderr(Stdio::from(guest_err))
        .spawn()
        .map_err(|e| format!("spawning guest game binary {}: {e}", bin.display()))?;
    eprintln!(
        "[2proc] guest pid={} log={}",
        guest.id(),
        guest_log_path.display()
    );

    let mut host = testing_4::ProductionGameApp::new(&wasm, &params, 0).await;

    // Both processes deploy the same fresh contract key, so a `Put` race (two disjoint replicas) can
    // only reconcile via freenet's ~5-minute InterestSync anti-entropy. The timeout must sit above
    // that window for the test to be reliable rather than flaky.
    host.wait_for_roster_len(2, Duration::from_secs(380))
        .await
        .map_err(|e| format!("host+guest subprocess should join the same contract: {e}"))?;
    host.wait_for_box_count(2, Duration::from_secs(60))
        .await
        .map_err(|e| format!("host should render one box per engine player: {e}"))?;

    let initial_x = host
        .own_box_position()
        .ok_or("host engine never positioned its own box")?
        .x;

    host.press_key(KeyCode::KeyD);
    for _ in 0..400 {
        host.tick();
        tokio::time::sleep(Duration::from_millis(STEP_DELAY_MS)).await;
    }
    host.release_key(KeyCode::KeyD);
    for _ in 0..200 {
        host.tick();
        tokio::time::sleep(Duration::from_millis(STEP_DELAY_MS)).await;
    }

    let div = host.divergence_count();
    let host_hash = host.state_hash();
    let moved = host
        .own_box_position()
        .map(|now| (now.x - initial_x).abs() > 10.0)
        .unwrap_or(false);

    let _ = guest.kill();
    let _ = guest.wait();

    assert_ne!(
        host_hash, 0,
        "host must have a state hash after the cross-process session (ensures the determinism gate is non-vacuous)"
    );
    assert_eq!(
        div, 0,
        "two separate processes converged on identical state hashes: found {div} divergence(s) on the host's final tick"
    );
    assert!(
        moved,
        "host's own box moved under engine authority once the guest joined (cross-process lockstep)"
    );
    Ok(())
}

#[derive(Serialize)]
struct Params {
    namespace: [u8; 32],
    max_members: u16,
}
// no test_usage necessary — full two-process live determinism gate (needs the binary present)
