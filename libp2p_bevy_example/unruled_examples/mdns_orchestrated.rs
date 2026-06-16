// This file is in `unruled_examples/` — it is EXEMPT from the project's
// syntax and architecture rules in `.agents/rules/`. Only files inside
// `unruled_examples/` share this exemption.
//
// OBJECTIVE: Same as mdns_separate_process_self_spawn.rs — cross-process
// mDNS discovery + bidirectional Ping over gossipsub — but uses the
// ProcessOrchestrator from this directory to handle process spawning,
// pipe management, output collection, and assertion helpers. This
// demonstrates the orchestrator's ability to reduce test complexity.
//
// Original:     149 lines, 56 non-blank assertion/spawn lines
// Orchestrated:  54 lines, 14 non-blank assertion/spawn lines

#[path = "utils.rs"]
mod utils;

use std::thread;
use std::time::Duration;

use bevy_p2p_app::p2p::testing::ProcessOrchestrator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("ORCH_PEER").is_ok() {
        return utils::run_mdns_peer();
    }

    tracing_subscriber::fmt::init();

    let mut orch = ProcessOrchestrator::new(Duration::from_secs(60))?;
    orch.spawn("A")?;
    // Stagger so A's mDNS listener is up before B starts
    thread::sleep(Duration::from_secs(2));
    orch.spawn("B")?;
    let output = orch.collect();

    let pid_a = output.peer_id("A").expect("Child A should report READY");
    let pid_b = output.peer_id("B").expect("Child B should report READY");

    assert!(
        output.has_discovered("A", &pid_b),
        "Child A should have discovered B ({pid_b})"
    );
    assert!(
        output.has_discovered("B", &pid_a),
        "Child B should have discovered A ({pid_a})"
    );
    assert!(
        output.has_connected("A", &pid_b),
        "Child A should have connected to B ({pid_b})"
    );
    assert!(
        output.has_connected("B", &pid_a),
        "Child B should have connected to A ({pid_a})"
    );
    assert!(output.has_sent_ping("A"));
    assert!(output.has_sent_ping("B"));
    assert!(
        output.has_got_ping("A", &pid_b),
        "Child A should have received a Ping from B ({pid_b})"
    );
    assert!(
        output.has_got_ping("B", &pid_a),
        "Child B should have received a Ping from A ({pid_a})"
    );
    assert!(output.has_success("A"));
    assert!(output.has_success("B"));

    println!(
        "\nSUCCESS: orchestrated — cross-process mDNS discovery \
         + bidirectional Ping"
    );
    Ok(())
}
