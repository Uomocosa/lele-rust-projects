// This file is in `unruled_examples/` — it is EXEMPT from the project's
// syntax and architecture rules in `.agents/rules/`. Only files inside
// `unruled_examples/` share this exemption. Refactor to follow the rules
// before promoting to `examples/` or `src/`.
//
// OBJECTIVE: Same as mdns_separate_process_self_spawn.rs (cross-process mDNS
// discovery + bidirectional Ping, faithfully simulating real game usage), but
// uses the `duct` crate instead of raw std::process::Command for process
// orchestration. duct provides ergonomic process building, automatic pipe
// management, and BufRead via .reader(). Both variants must pass before
// promoting any mDNS code out of unruled/.

#[path = "utils.rs"]
mod utils;

use std::collections::HashSet;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use duct::cmd;

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

    // --- Stderr pipes (created before children so they stay alive) ---
    let (reader_a_err, writer_a_err) = os_pipe::pipe()?;
    let (reader_b_err, writer_b_err) = os_pipe::pipe()?;

    // --- Spawn child A via duct + os_pipe (so we can kill later) ---
    let (reader_a, writer_a) = os_pipe::pipe()?;
    let child_a = cmd!(exe.clone())
        .env("MDNS_PEER", "1")
        .stdout_file(writer_a)
        .stderr_file(writer_a_err)
        .unchecked()
        .start()?;

    std::thread::sleep(Duration::from_secs(2));

    // --- Spawn child B via duct + os_pipe ---
    let (reader_b, writer_b) = os_pipe::pipe()?;
    let child_b = cmd!(exe)
        .env("MDNS_PEER", "1")
        .stdout_file(writer_b)
        .stderr_file(writer_b_err)
        .unchecked()
        .start()?;

    // --- Reader threads → shared mpsc (using shared helper) ---
    let (tx, rx) = mpsc::channel::<String>();
    let h_a_out = utils::spawn_stdout_reader("A", reader_a, tx.clone());
    let h_b_out = utils::spawn_stdout_reader("B", reader_b, tx.clone());
    let h_a_err = utils::spawn_stdout_reader("A:stderr", reader_a_err, tx.clone());
    let h_b_err = utils::spawn_stdout_reader("B:stderr", reader_b_err, tx);

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

    // --- Kill children, wait for reaping, join reader threads ---
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

    println!("\nSUCCESS: duct — cross-process mDNS discovery + bidirectional Ping");
    Ok(())
}
