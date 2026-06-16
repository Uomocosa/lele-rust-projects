// This file is in `unruled_examples/` — it is EXEMPT from the project's
// syntax and architecture rules in `.agents/rules/`. Only files inside
// `unruled_examples/` share this exemption. Refactor to follow the rules
// before promoting to `examples/` or `src/`.
//
// OBJECTIVE: Cross-process mDNS discovery + bidirectional Ping over
// gossipsub, faithfully simulating real game usage. Two child processes
// each enable mDNS, discover ALL LAN responders (not just each other),
// dial every discovered peer, and exchange Message::Ping over gossipsub.
// The orchestrator validates that the sibling pair (A↔B) discovered,
// connected, and exchanged pings — regardless of other mDNS responders
// on the network. This is the primary integration test for real mDNS
// networking in a real-LAN scenario.

#[path = "utils.rs"]
mod utils;

use std::collections::HashSet;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

const TIMEOUT: Duration = Duration::from_secs(60);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("MDNS_PEER").is_ok() {
        return utils::run_mdns_peer();
    }
    run_orchestrator()
}

fn run_orchestrator() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let exe = std::env::current_exe()?;

    // --- Spawn child A ---
    let mut child_a = Command::new(&exe)
        .env("MDNS_PEER", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout_a = child_a.stdout.take().unwrap();

    // Stagger so A starts listening before B starts
    std::thread::sleep(Duration::from_secs(2));

    // --- Spawn child B ---
    let mut child_b = Command::new(&exe)
        .env("MDNS_PEER", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout_b = child_b.stdout.take().unwrap();

    // --- Stderr pipes ---
    let stderr_a = child_a.stderr.take().unwrap();
    let stderr_b = child_b.stderr.take().unwrap();

    // --- Reader threads → shared mpsc (stdout) ---
    let (tx, rx) = mpsc::channel::<String>();
    let h_a_out = utils::spawn_stdout_reader("A", stdout_a, tx.clone());
    let h_b_out = utils::spawn_stdout_reader("B", stdout_b, tx.clone());

    // Also forward stderr lines for debugging
    let h_a_err = utils::spawn_stdout_reader("A:stderr", stderr_a, tx.clone());
    let h_b_err = utils::spawn_stdout_reader("B:stderr", stderr_b, tx);

    // --- Collect Evt: lines until timeout ---
    let deadline = Instant::now() + TIMEOUT;
    let mut seen: HashSet<String> = HashSet::new();
    let mut pid_a: Option<String> = None;
    let mut pid_b: Option<String> = None;

    while Instant::now() < deadline {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(line) => {
                println!("[child] {}", line);
                let trimmed = line.trim().to_string();
                if let Some(pid) = trimmed.strip_prefix("A:Evt:READY:") {
                    pid_a = Some(pid.to_string());
                } else if let Some(pid) = trimmed.strip_prefix("B:Evt:READY:") {
                    pid_b = Some(pid.to_string());
                }
                seen.insert(trimmed);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    // --- Kill and reap children ---
    let _ = child_a.kill();
    let _ = child_a.wait();
    let _ = child_b.kill();
    let _ = child_b.wait();
    let _ = h_a_out.join();
    let _ = h_b_out.join();
    let _ = h_a_err.join();
    let _ = h_b_err.join();

    // --- Assertions (sibling-pair-exists semantics) ---
    let pid_a = pid_a.expect("Child A should report READY with peer ID");
    let pid_b = pid_b.expect("Child B should report READY with peer ID");

    // Both must have discovered each other via mDNS (among possibly other peers)
    assert!(
        seen.contains(&format!("A:Evt:DISCOVERED:{pid_b}")),
        "Child A should have discovered B's peer ID ({pid_b})"
    );
    assert!(
        seen.contains(&format!("B:Evt:DISCOVERED:{pid_a}")),
        "Child B should have discovered A's peer ID ({pid_a})"
    );

    // Both must have connected to each other
    assert!(
        seen.contains(&format!("A:Evt:CONNECTED:{pid_b}")),
        "Child A should have connected to B ({pid_b})"
    );
    assert!(
        seen.contains(&format!("B:Evt:CONNECTED:{pid_a}")),
        "Child B should have connected to A ({pid_a})"
    );

    // Both must have sent a Ping
    assert!(seen.contains("A:Evt:SENT_PING"));
    assert!(seen.contains("B:Evt:SENT_PING"));

    // Both must have received at least one Ping (with sender info)
    let a_got_ping_b = seen.contains(&format!("A:Evt:GOT_PING:{}", pid_b));
    let b_got_ping_a = seen.contains(&format!("B:Evt:GOT_PING:{}", pid_a));
    assert!(
        a_got_ping_b,
        "Child A should have received a Ping from B ({})",
        pid_b
    );
    assert!(
        b_got_ping_a,
        "Child B should have received a Ping from A ({})",
        pid_a
    );

    // Both must report success
    assert!(seen.contains("A:Evt:SUCCESS"));
    assert!(seen.contains("B:Evt:SUCCESS"));

    println!("\nSUCCESS: self-spawn — cross-process mDNS discovery + bidirectional Ping");
    Ok(())
}
