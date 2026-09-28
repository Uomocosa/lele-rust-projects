use std::process::{Child, Command};
use std::time::{Duration, Instant};

pub fn stop_app(child: &mut Child) {
    signal_group(child.id(), "-TERM");
    let deadline = Instant::now().checked_add(Duration::from_secs(5));
    while matches!(child.try_wait(), Ok(None)) {
        if deadline.is_none_or(|limit| Instant::now() >= limit) {
            signal_group(child.id(), "-KILL");
            let _ = child.kill();
            let _ = child.wait();
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

// needed helper: signal the whole process group so `cargo run` children die too
fn signal_group(pid: u32, signal: &str) {
    let _ = Command::new("kill")
        .args([signal, "--", &format!("-{pid}")])
        .status();
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::stop_app;
    use crate::process;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let command = vec!["sleep".to_string(), "30".to_string()];
        let mut child = process::spawn_app(
            &command,
            dir.path(),
            &BTreeMap::new(),
            &dir.path().join("l"),
        )
        .unwrap();
        stop_app(&mut child);
        assert!(child.try_wait().unwrap().is_some());
    }
}
